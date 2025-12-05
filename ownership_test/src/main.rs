use my_math::math::*;

fn main() {
    // BASIC OWNERSHIP:
    let mut string_val = String::from("OG stuff");

    /*
     * The following assignment performs a shallow copy (pointer, len, cap),
     * which means they both point to the same memory. If left like this,
     * when both variables go out of scope, the "drop" (free) operation would
     * be performed twice on the same memory which can cause issues - **double free error**.
     * This is WHY Rust drops the first one and transfers the ownership of that variable.
     */
    // let string_val_copy = string_val;

    /*
     * Value has to be cloned because otherwise the ownership is transfered
     * and the variable will dissapear after the function executes.
     * This is only the case because Strings are stored in the heap.
     */
    process_string(string_val.clone());
    println!("string_val = '{}'", string_val);

    string_val = process_string_return(string_val);
    println!("string_val = '{}'", string_val);

    let int_val = 10;
    process_int(int_val);
    println!("int_val = {}", int_val);

    // BORROWING:
    process_string_ref(&string_val);
    println!("borrowed string_val = '{}'", string_val);

    modify_string_ref(&mut string_val);
    println!("borrowed string_val = '{}'", string_val);

    let add_result = add(2, 5);
    println!("add_result = '{}'", add_result);
}

fn process_string(string_arg: String) {
    /*
     * If the argument will be modified inside the function,
     * then it needs to be mut but those changes will still
     * be ONLY inside this context.
     */
    println!("processing string {} ...", string_arg);
}

fn process_string_return(mut string_arg: String) -> String {
    println!("processing string return {} ...", string_arg);
    string_arg = String::from("new stuff");
    string_arg
}

fn process_int(mut int_arg: u8) {
    println!("processing int {} ...", int_arg);
    int_arg += 1;
    println!("processed int_arg = {}", int_arg);
}

fn process_string_ref(string_arg: &String) {
    println!("processing string ref {} ...", string_arg);
}

fn modify_string_ref(string_arg: &mut String) {
    println!("processing string ref '{}' ...", string_arg);
    string_arg.push_str(" with addition");
}
