mod custom_structs;
mod linux_function;


fn main(){

    
    let mut test = linux_function::query_directory_in_file_path("123", "/home/isaac/Downloads");
    test.item_path = "test1".to_string();

    println!("{:#?}", test)


}

