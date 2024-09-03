use crate::models::general::llm::Message;

pub fn extend_ai_function(ai_func:fn(&str) -> &'static str, func_input:&str)  {
    let ai_function_str = ai_func(func_input);
    // extend the string to encourage only printing the output
    let msg:String = format!("FUNCTION {}\
    INSTRUCTION: You are a function printer. You ONLY print the results of functions.\
    Nothing else. No commentary. Here is the input to the function: {}", ai_function_str, func_input);

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai_functions::ai_func_managing::convert_user_input_to_goal;

    #[test]
    fn tests_extending_ai_function() {
        extend_ai_function( convert_user_input_to_goal,"dummy variable");
    }
}