
use crate::models::general::llm::MessageAI;

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
}