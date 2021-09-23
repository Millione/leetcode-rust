#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

mod fetcher;
mod helper;

use crate::fetcher::get_problems_async;
use crate::helper::{
    build_desc, insert_return_in_code, parse_discuss_link, parse_extra_use, parse_problem_link,
};
use fetcher::Problems;
use once_cell::sync::OnceCell;
use regex::Regex;
use std::path::Path;
use tokio::fs;
use tokio::fs::File;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;

static PROBLEMS: OnceCell<Problems> = OnceCell::new();

/// main() helps to generate the submission template.rs
#[tokio::main]
async fn main() {
    let problems = get_problems_async().await.unwrap();
    PROBLEMS.set(problems).unwrap();
    println!("Welcome to leetcode-rust system.\n");
    let content = fs::read_to_string("./src/problems/mod.rs").await.unwrap();
    #[allow(unused_mut)]
    let mut initialized_ids = get_initialized_ids(content);
    loop {
        println!(
            "Please enter a frontend problem \"$id\", \n\
            or \"random\" to generate a random one, \n\
            or \"solve $id\" to move problem to solutions/ \n"
        );
        #[allow(unused_assignments)]
        let mut id: u32 = 0;
        let mut id_arg = String::new();
        std::io::stdin()
            .read_line(&mut id_arg)
            .expect("Failed to read line");
        let id_arg = id_arg.trim();

        let random_pattern = Regex::new(r"^random$").unwrap();
        let solving_pattern = Regex::new(r"^solve (\d+)$").unwrap();

        if random_pattern.is_match(id_arg) {
            println!("You select random mode.");
            id = generate_random_id(&initialized_ids);
            println!("Generate random problem: {}", &id);
        } else if solving_pattern.is_match(id_arg) {
            // solve a problem
            // move it from problems/ to solutions/
            id = solving_pattern
                .captures(id_arg)
                .unwrap()
                .get(1)
                .unwrap()
                .as_str()
                .parse()
                .unwrap();
            deal_solving(&id).await;
            break;
        } else {
            id = id_arg
                .parse::<u32>()
                .unwrap_or_else(|_| panic!("not a number: {}", id_arg));
            if initialized_ids.contains(&id) {
                println!(
                    "The problem {} you chose has been initialized in problems/",
                    id
                );
                continue;
            }
        }
        deal_problem(id).await;
        break;
    }
}

fn generate_random_id(except_ids: &[u32]) -> u32 {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    loop {
        let res: u32 = rng.gen_range(1..=2021);
        if !except_ids.contains(&res) {
            return res;
        }
        println!(
            "Generate a random num ({}), but it is invalid (the problem may have been solved \
             or may have no rust version). Regenerate..",
            res
        );
    }
}

fn get_initialized_ids(content: String) -> Vec<u32> {
    let id_pattern = Regex::new(r"p(\d{4})_").unwrap();
    id_pattern
        .captures_iter(&content)
        .map(|x| x.get(1).unwrap().as_str().parse().unwrap())
        .collect()
}

async fn deal_solving(id: &u32) {
    let problem = fetcher::get_problem_by_id(*id).await.unwrap();
    let file_name = format!(
        "p{:04}_{}",
        problem.question_id,
        problem.title_slug.replace("-", "_")
    );
    let file_path = Path::new("./src/problems").join(format!("{}.rs", file_name));
    // check problems/ existence
    if !file_path.exists() {
        panic!("Problem {} does not exist", id);
    }
    // check solutions/ no existence
    let solution_name = format!(
        "s{:04}_{}",
        problem.question_id,
        problem.title_slug.replace("-", "_")
    );
    let solution_path = Path::new("./src/solutions").join(format!("{}.rs", solution_name));
    if solution_path.exists() {
        panic!("Solution {} exists", id);
    }
    // rename/move file
    fs::rename(file_path, solution_path).await.unwrap();
    // remove from problems/mod.rs
    let mod_file = "./src/problems/mod.rs";
    let target_line = format!("mod {};", file_name);
    let mut mod_lines = vec![];
    let mut lines = BufReader::new(File::open(mod_file).await.unwrap()).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        if line != target_line {
            mod_lines.push(line);
        }
    }
    fs::write(mod_file, mod_lines.join("\n")).await.unwrap();
    // insert into solutions/mod.rs
    insert_mod_file("./src/solutions/mod.rs", &solution_name).await;
    insert_readme_file(*id, solution_name, problem.title_slug, problem.difficulty).await;
}

#[allow(dead_code)]
async fn deal_problems(initialized_ids: Vec<u32>) {
    let mut handles = vec![];
    for problem_stat in &PROBLEMS.get().unwrap().stat_status_pairs {
        if initialized_ids.contains(&problem_stat.stat.frontend_question_id) {
            continue;
        }
        handles.push(tokio::spawn({
            async move { deal_problem(problem_stat.stat.frontend_question_id).await }
        }));
    }
    futures::future::join_all(handles).await;
}

async fn deal_problem(id: u32) {
    let problem = fetcher::get_problem_by_id(id).await.unwrap_or_else(|| {
        panic!(
            "Error: failed to get problem #{} \
             (The problem may be paid-only or may not be exist).",
            id
        )
    });
    let code = problem.code_definition.iter().find(|&d| d.value == *"rust");
    if code.is_none() {
        println!("Problem {} has no rust version.", problem.question_id);
        return;
    }
    let code = code.unwrap();

    let file_name = format!(
        "p{:04}_{}",
        problem.question_id,
        problem.title_slug.replace("-", "_")
    );
    let file_path = Path::new("./src/problems").join(format!("{}.rs", file_name));
    if file_path.exists() {
        panic!("Problem {} already initialized", problem.question_id);
    }

    let template = fs::read_to_string("./template.rs").await.unwrap();
    let source = template
        .replace("__PROBLEM_TITLE__", &problem.title)
        .replace("__PROBLEM_DESC__", &build_desc(&problem.content))
        .replace(
            "__PROBLEM_DEFAULT_CODE__",
            &insert_return_in_code(&problem.return_type, &code.default_code),
        )
        .replace("__PROBLEM_ID__", &format!("{}", problem.question_id))
        .replace("__EXTRA_USE__", &parse_extra_use(&code.default_code))
        .replace("__PROBLEM_LINK__", &parse_problem_link(&problem))
        .replace("__DISCUSS_LINK__", &parse_discuss_link(&problem));

    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&file_path)
        .await
        .unwrap();

    file.write_all(source.as_bytes()).await.unwrap();
    drop(file);
    insert_mod_file("./src/problems/mod.rs", &file_name).await;
}

async fn insert_mod_file(path: impl AsRef<Path>, name: &str) {
    let mut mod_file = fs::OpenOptions::new()
        .append(true)
        .open(path)
        .await
        .unwrap();
    mod_file
        .write_all(format!("mod {};\n", name).as_bytes())
        .await
        .unwrap();
}

async fn insert_readme_file(id: u32, solution_name: String, title_slug: String, level: String) {
    let mut readme_file = fs::OpenOptions::new()
        .append(true)
        .open("./README.md")
        .await
        .unwrap();
    readme_file
        .write_all(
            format!(
                "<tr>
      <td>{}</td>
      <td><a href=\"./src/solutions/{}\"> {}</a></td>
      <td>{}</td>
    </tr>\n    ",
                id, solution_name, title_slug, level
            )
            .as_bytes(),
        )
        .await
        .unwrap();
}
