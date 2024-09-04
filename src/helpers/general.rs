use crate::helpers::command_line::PrintCommand;
use crate::models::general::llm::MessageAI;
use crate::apis::call_request::call_gpt;

pub fn extend_ai_function(ai_func:fn(&str) -> &'static str, func_input:&str) -> MessageAI {
    let ai_function_str = ai_func(func_input);
    // extend the string to encourage only printing the output
    let msg:String = format!("FUNCTION {}
    INSTRUCTION: You are a function printer. You ONLY print the results of functions.
    Nothing else. No commentary. Here is the input to the function: {}", ai_function_str, func_input);

    // Return the message
    MessageAI{
        role:"system".to_string(),
        content:msg
    }

}
//performs call to our llm
pub async fn ai_task_request(
    msg_context:String,
    agent_position:&str,agent_operation:&str,
    function_pass:for<'a> fn(&'a str) -> &'static str
) -> String{
    // extend ai function
    let extended_msg:MessageAI = extend_ai_function(function_pass,&msg_context);
    // print current status
    PrintCommand::AICall.print_agent_message(agent_position,agent_operation);
    // Get LLM response
    let llm_response_res:Result<String,Box<dyn std::error::Error + Send>> = call_gpt(vec![extended_msg.clone()]).await;
    // handle success
    match llm_response_res {
        Ok(llm_resp) => llm_resp,
        Err(_) => call_gpt(vec![extended_msg.clone()])
            .await
            .expect("Failed twice to call OpenAI")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai_functions::ai_func_managing::convert_user_input_to_goal;

    #[test]
    fn tests_extending_ai_function() {
        let extended_ai_func = extend_ai_function( convert_user_input_to_goal,"Create a Web3 project");
        dbg!(&extended_ai_func);
        assert_eq!(extended_ai_func.role, "system".to_string());
    }

    #[tokio::test]
    async fn tests_ai_task_request() {
        let ai_func_param: String =
            "Build me a webserver for making stock price api requests.".to_string();

        let res: String = ai_task_request(
            ai_func_param,
            "Managing Agent",
            "Defining user requirements",
            convert_user_input_to_goal,
        ).await;

        assert!(res.len() > 20);
        dbg!(res);
    }
}