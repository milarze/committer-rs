pub fn generate_prompt(diff: String, scopes: Vec<String>, context: Option<String>) -> String {
    let scopes = scopes.join("\n");
    if let Some(context) = context {
        commit_message_and_body_prompt()
            .replace("{{DIFF}}", &diff)
            .replace("{{SCOPES}}", &scopes)
            .replace("{{CONTEXT}}", &context)
    } else {
        commit_message_only_prompt()
            .replace("{{DIFF}}", &diff)
            .replace("{{SCOPES}}", &scopes)
    }
}

fn commit_message_only_prompt() -> String {
    "
    You are an experienced software developer tasked with creating a commit message based on a git diff. Your goal is to produce a clear, concise, and informative commit message.

    First, carefully analyze the following git diff:

    <git_diff>
    {{DIFF}}
    </git_diff>

    Here are the available scopes (if any):

    <scopes>
    {{SCOPES}}
    </scopes>



    2. Adhere to these message guidelines:
       - Keep the summary under 70 characters
       - Use imperative, present tense (e.g., \"add\" not \"added\" or \"adds\")
       - Do not end the summary with a period
       - Be concise but descriptive

    3. Format the commit message as follows:
       - If a scope is available: <type>(<scope>): <description>
       - If no scope is available: <type>: <description>


    Please follow these instructions to generate the commit message:

    if such and such changes its docs b ut if commit also has such then its a feat

    1. Analyze the git diff and determine the most appropriate commit type from the following options:
       - feat: A new feature these changes ARE NOT in the docs directory
       - fix: A bug fix these changes ARE NOT in the docs directory
       - docs: Documentation only changes for example changes that happen in docs directory
       - style: Changes that do not affect the code execution (e.g., white-space, formatting, missing semi-colons, etc.) usually done by linters
       - refactor: 
          - A code change that does not change the behaviour of the code at all, just how the code is written executed, 
          - modifing error messages is not a refactor
          - if parts of the code is removed and no alternative in place IT IS NOT A REFACTOR
       - perf: A code change that improves performance
       - test: Adding missing tests or correcting existing tests
       - build: Changes that affect the build system or external dependencies
       - ci: 
          - Changes to the CI configuration files and scripts
          - these changes should not change the way that the code is built otherwise its build


    Respond ONLY with the commit message line, nothing else.
    ".to_string()
}

fn commit_message_and_body_prompt() -> String {
    "
    You are an experienced software developer tasked with creating a commit message based on a git diff. Your goal is to produce a clear, concise, and informative commit message.

    First, carefully analyze the following git diff:

    <git_diff>
    {{DIFF}}
    </git_diff>

    Here are the available scopes (if any):

    <scopes>
    {{SCOPES}}
    </scopes>

    <user_context>
    {{CONTEXT}}
    </user_context>

    Please follow these instructions to generate the commit message:

    1. Analyze the git diff and determine the most appropriate commit type from the following options:
       - feat: A new feature
       - fix: A bug fix
       - docs: Documentation only changes
       - style: Changes that do not affect the meaning of the code
       - refactor: A code change that neither fixes a bug nor adds a feature
       - perf: A code change that improves performance
       - test: Adding missing tests or correcting existing tests
       - chore: Changes to the build process or auxiliary tools and libraries


    2. Adhere to these message guidelines:
       - Keep the summary under 70 characters
       - Use imperative, present tense (e.g., \"add\" not \"added\" or \"adds\")
       - Do not end the summary with a period
       - Be concise but descriptive

    3. Format the commit message as follows:
       - If a scope is available: <type>(<scope>): <description>
       - If no scope is available: <type>: <description>

    4 Body Guidelines:
       - Add a blank line between summary and body
       - Use the body to explain why the change was made, incorporating the user's context, defined in <user_context>
       - Wrap each line in the body at 80 characters maximum
       - Break the body into multiple paragraphs if needed

       [Your concise commit message in the specified format]
       [blank line]
       [Your detailed commit message body]

    Respond ONLY with the commit message text (message and body), nothing else.
    ".to_string()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_commit_message_only_prompt() {
        let diff = "diff --git a/src/main.rs b/src/main.rs\nindex 5e6f3a1..b6f3a1b 100644\n--- a/src/main.rs\n+++ b/src/main.rs\n@@ -1,3 +1,3 @@\n fn main() {\n-    println!(\"Hello, world!\");\n+    println!(\"Hello, Rust!\");\n }".to_string();
        let scopes = vec!["src".to_string(), "main".to_string()];
        let prompt = generate_prompt(diff, scopes, None);
        assert!(prompt.contains("src"));
        assert!(prompt.contains("fn main()"));
    }

    #[test]
    fn test_commit_message_and_body_prompt() {
        let context = Some("This is some context".to_string());
        let diff = "diff --git a/src/main.rs b/src/main.rs\nindex 5e6f3a1..b6f3a1b 100644\n--- a/src/main.rs\n+++ b/src/main.rs\n@@ -1,3 +1,3 @@\n fn main() {\n-    println!(\"Hello, world!\");\n+    println!(\"Hello, Rust!\");\n }".to_string();
        let scopes = vec!["src".to_string(), "main".to_string()];
        let prompt = generate_prompt(diff, scopes, context);
        assert!(prompt.contains("src"));
        assert!(prompt.contains("fn main()"));
        assert!(prompt.contains("This is some context"));
    }
}
