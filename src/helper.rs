use crate::fetcher::Problem;
use regex::Regex;

pub fn parse_extra_use(code: &str) -> String {
    let mut extra_use_line = String::new();
    // a linked-list problem
    if code.contains("pub struct ListNode") {
        extra_use_line.push_str("\nuse crate::util::linked_list::{ListNode, to_list};")
    }
    if code.contains("pub struct TreeNode") {
        extra_use_line.push_str("\nuse crate::util::tree::{TreeNode, to_tree};")
    }
    if code.contains("pub struct Point") {
        extra_use_line.push_str("\nuse crate::util::point::Point;")
    }
    extra_use_line
}

pub fn parse_problem_link(problem: &Problem) -> String {
    format!("https://leetcode.com/problems/{}/", problem.title_slug)
}

pub fn parse_discuss_link(problem: &Problem) -> String {
    format!(
        "https://leetcode.com/problems/{}/discuss/?currentPage=1&orderBy=most_votes&query=",
        problem.title_slug
    )
}

pub fn insert_return_in_code(return_type: &str, code: &str) -> String {
    let re = Regex::new(r"\{[ \n]+}").unwrap();
    match return_type {
        "ListNode" => re
            .replace(code, "{\n        Some(Box::new(ListNode::new(0)))\n    }")
            .to_string(),
        "ListNode[]" => re.replace(code, "{\n        vec![]\n    }").to_string(),
        "TreeNode" => re
            .replace(
                code,
                "{\n        Some(Rc::new(RefCell::new(TreeNode::new(0))))\n    }",
            )
            .to_string(),
        "boolean" => re.replace(code, "{\n        false\n    }").to_string(),
        "character" => re.replace(code, "{\n        '0'\n    }").to_string(),
        "character[][]" => re.replace(code, "{\n        vec![]\n    }").to_string(),
        "double" => re.replace(code, "{\n        0f64\n    }").to_string(),
        "double[]" => re.replace(code, "{\n        vec![]\n    }").to_string(),
        "int[]" => re.replace(code, "{\n        vec![]\n    }").to_string(),
        "integer" => re.replace(code, "{\n        0\n    }").to_string(),
        "integer[]" => re.replace(code, "{\n        vec![]\n    }").to_string(),
        "integer[][]" => re.replace(code, "{\n        vec![]\n    }").to_string(),
        "list<String>" => re.replace(code, "{\n        vec![]\n    }").to_string(),
        "list<TreeNode>" => re.replace(code, "{\n        vec![]\n    }").to_string(),
        "list<boolean>" => re.replace(code, "{\n        vec![]\n    }").to_string(),
        "list<double>" => re.replace(code, "{\n        vec![]\n    }").to_string(),
        "list<integer>" => re.replace(code, "{\n        vec![]\n    }").to_string(),
        "list<list<integer>>" => re.replace(code, "{\n        vec![]\n    }").to_string(),
        "list<list<string>>" => re.replace(code, "{\n        vec![]\n    }").to_string(),
        "list<string>" => re.replace(code, "{\n        vec![]\n    }").to_string(),
        "null" => code.to_string(),
        "string" => re
            .replace(code, "{\n        String::new()\n    }")
            .to_string(),
        "string[]" => re.replace(code, "{\n        vec![]\n    }").to_string(),
        "void" => code.to_string(),
        "NestedInteger" => code.to_string(),
        "Node" => code.to_string(),
        _ => code.to_string(),
    }
}

pub fn build_desc(content: &str) -> String {
    // TODO: fix this shit
    content
        .replace("<strong>", "")
        .replace("</strong>", "")
        .replace("<em>", "")
        .replace("</em>", "")
        .replace("</p>", "")
        .replace("<p>", "")
        .replace("<b>", "")
        .replace("</b>", "")
        .replace("<pre>", "")
        .replace("</pre>", "")
        .replace("<ul>", "")
        .replace("</ul>", "")
        .replace("<li>", "")
        .replace("</li>", "")
        .replace("<code>", "")
        .replace("</code>", "")
        .replace("<i>", "")
        .replace("</i>", "")
        .replace("<sub>", "")
        .replace("</sub>", "")
        .replace("</sup>", "")
        .replace("<sup>", "^")
        .replace("&nbsp;", " ")
        .replace("&gt;", ">")
        .replace("&lt;", "<")
        .replace("&quot;", "\"")
        .replace("&minus;", "-")
        .replace("&#39;", "'")
        .replace("\n\n", "\n")
        .replace("\n", "\n * ")
        .replace("<font face=\"monospace\">", "")
        .replace("</font>", "")
        .replace("<u>", "")
        .replace("</u>", "")
        .replace("<ol>", "")
        .replace("</ol>", "")
        .replace("<s>", "")
        .replace("</s>", "")
        .replace("<blockquote>", "")
        .replace("</blockquote>", "")
}
