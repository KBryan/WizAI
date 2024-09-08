use ai_functions::ai_function;

#[ai_function]
pub fn print_frontend_webapp_code(_project_description_and_template: &str) {
    /// INPUT: Takes in a PROJECT_DESCRIPTION and CODE_TEMPLATE for a frontend web app build
    /// IMPORTANT: The frontend code is ONLY an example. If the Project Description requires it, make as many changes as you like.
    /// FUNCTION: Takes an existing set of code marked as CODE_TEMPLATE and updates or re-writes it to work for the purpose in the PROJECT_DESCRIPTION
    /// IMPORTANT: The following libraries are assumed to be available:
    ///   HTMLX, Javascript, CSS3, TailwindCSS, Bootstrap
    /// No other external libraries should be used unless specified in the description.
    /// OUTPUT: Print ONLY the code, nothing else. This function ONLY prints code.
    println!(OUTPUT)
}

#[ai_function]
pub fn print_improved_frontend_webapp_code(_project_description_and_template: &str) {
    /// INPUT: Takes in a PROJECT_DESCRIPTION and CODE_TEMPLATE for a frontend web app build
    /// FUNCTION: Performs the following tasks:
    ///   1. Removes any bugs in the frontend code and adds minor additional functionality if required by the project.
    ///   2. Makes sure everything requested in the spec from a frontend standpoint was followed. If not, add the feature. No code should be implemented later. Everything should be written now.
    ///   3. ONLY writes the code. No commentary.
    /// IMPORTANT: The following libraries are assumed to be available:
    ///   React, Vue.js, Angular, TailwindCSS, Bootstrap
    /// OUTPUT: Prints ONLY the improved code.
    println!(OUTPUT)
}

#[ai_function]
pub fn print_fixed_frontend_code(_broken_code_with_bugs: &str) {
    /// INPUT: Takes in BROKEN_CODE and the list of ERROR_BUGS
    /// FUNCTION: Removes bugs from the frontend code.
    /// IMPORTANT: Only prints out the new and improved code. No commentary or anything else.
    println!(OUTPUT)
}

#[ai_function]
pub fn print_frontend_ui_components(_code_input: &str) {
    /// INPUT: Takes in FRONTEND UI CODE_INPUT based on React, Vue, or Angular
    /// FUNCTION: Prints out a list of UI components and their respective props and state values
    /// LOGIC: Script analyzes the input code and can categorize into the following object keys:
    ///   "component": The name of the component
    ///   "props": The properties passed into the component
    ///   "state": The state values maintained within the component
    /// IMPORTANT: Only prints out the component details. No commentary or anything else.
    /// OUTPUT EXAMPLE:
    /// INPUT_CODE:
    /// ...
    /// function Button({ label, onClick }) {
    ///   const [loading, setLoading] = useState(false);
    ///   return (
    ///     <button onClick={onClick}>{loading ? "Loading..." : label}</button>
    ///   );
    /// }
    /// ...
    /// PRINTS JSON FORMATTED OUTPUT:
    /// [
    ///   {
    ///     "component": "Button",
    ///     "props": {
    ///       "label": "string",
    ///       "onClick": "function"
    ///     },
    ///     "state": {
    ///       "loading": "boolean"
    ///     }
    ///   },
    ///   ... // etc.
    /// ]
    println!(OUTPUT)
}
