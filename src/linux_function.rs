use run_script::{
    ScriptOptions,
    run_script
};
use std::{
    sync::mpsc::channel,
    thread,
};

use crate::{
    custom_structs::{
        ItemMetaData,
        DirectoryInfo,
    },
};


pub fn query_item_path_metadata(password: &str, file_path: &str) -> ItemMetaData {
    let options = ScriptOptions::new();
    let command = format!(
r#"echo {} | sudo -S find '{}' -maxdepth 0 -printf '%TF %TH:%TM|%y|%s'"#, password, file_path);
    let output;

    loop {
        let operation = run_script!(
            &format!("{}", command),
            &vec![],
            &options
        );

        match operation {
            Ok(result_tuple) => {
                output = result_tuple.1;
                break;
            },
            Err(_err) => thread::yield_now()
        }
    }

    // println!("Item: {}, Output: {}", &file_path, &output);

    let splited_output = output.split("|").map(|argument: &str| argument.to_string()).collect::<Vec<String>>();

    ItemMetaData {
        item_last_modify_date: splited_output[0].to_owned(),
        item_type: match splited_output[1].as_ref() {
            "d" => "directory".to_owned(),
            _ => "file".to_owned()
        },
        item_size: splited_output[2].parse().unwrap()
    }

}

pub fn query_directory_in_file_path(password: &str, directory_path: &str) -> DirectoryInfo {
    let options = ScriptOptions::new();
    let command = format!(
r#"echo {} | sudo -S find '{}' -maxdepth 1 -not -path '*/\.*' -printf '%p\n'| sed '1d'"#, password, directory_path);
    
    // let (mut _code, mut output, mut _error) = (1, String::new(), String::new());
    let mut output;

    loop {
        let operation = run_script!(
            &format!("{}", command),
            &vec![],
            &options
        );

        match operation {
            Ok(result_tuple) => {
                output = result_tuple.1;
                break;
            },
            Err(_err) => thread::yield_now()
        }
    }



    // Fix Empty line without \n
    match output.is_empty() {
        true => (),
        false => output.truncate(output.len() -1 ),
    }


    match output.lines().count() {
        0 => {
            let display_name: String = directory_path.split("/").collect::<Vec<&str>>().last().unwrap().to_string();
            DirectoryInfo {
                item_path: display_name,
                item_metadata: query_item_path_metadata(password, directory_path),
                sub_path_items: vec![]
            }
        },
        1 => match query_item_path_metadata(password, &output).get_type().as_str() {
            "directory" => query_directory_in_file_path(password, &output),
            _ => {
                let mut full_dir_name_vec = output.split("/").map(|argument| argument.to_string()).collect::<Vec<String>>();
                
                let file_display_name: String = full_dir_name_vec.last().unwrap().to_string();
                full_dir_name_vec.remove(full_dir_name_vec.len() -1);

                let dir_display_name: String = full_dir_name_vec.last().unwrap().to_string();
                let full_previous_dir = full_dir_name_vec.join("/");

                DirectoryInfo {
                    item_path: dir_display_name,
                    item_metadata: query_item_path_metadata(password, &full_previous_dir),
                    sub_path_items: vec![
                        Box::new(
                            DirectoryInfo {
                                item_path: file_display_name,
                                item_metadata: query_item_path_metadata(password, &output),
                                sub_path_items: vec![]
                            }
                        )
                    ]
                }
            },
        }
        _ => {
                let mut vec_info: Vec<Box<DirectoryInfo>> = Vec::new();
                let mut thread_vec = Vec::new();
                let lines_output = output.lines().into_iter().map(|argument| argument.to_string()).collect::<Vec<String>>();
                let (sender, reciever) = channel::<Box<DirectoryInfo>>();   
    
                for each_line in lines_output {

                    let sender_clone = sender.clone();
                    let password_clone = password.to_owned();

                    thread_vec.push(
                        thread::spawn(
                            move || {
                                sender_clone.send(
                                        Box::new(
                                            match query_item_path_metadata(&password_clone, &each_line).get_type().as_str() {
                                                "directory" => query_directory_in_file_path(&password_clone, &each_line),
                                                _ => {
                                                    let display_name: String = each_line.split("/").collect::<Vec<&str>>().last().unwrap().to_string();
                                                    DirectoryInfo {
                                                        item_path: display_name,
                                                        item_metadata: query_item_path_metadata(&password_clone, &each_line),
                                                        sub_path_items: vec![]
                                                    }
                                                },
                                            }
                                        )
                                )
                                    .unwrap();
                            }
                        )
                    );
                    
                }
                
                for each_thread in thread_vec {
                    each_thread.join().unwrap();
                    vec_info.push(reciever.try_recv().unwrap());
                }

                // let mut vec_info: Vec<Box<DirectoryInfo>> = Vec::new();
                // for each_line in output.lines() {
                //     vec_info.push(
                //         Box::new(
                //             match query_item_path_metadata(password, each_line).get_type().as_str() {
                //                 "directory" => query_directory_in_file_path(password, each_line),
                //                 _ => {
                //                     let display_name: String = each_line.split("/").collect::<Vec<&str>>().last().unwrap().to_string();
                //                     DirectoryInfo {
                //                         item_path: display_name,
                //                         item_metadata: query_item_path_metadata(password, &each_line),
                //                         sub_path_items: vec![]
                //                     }
                //                 },
                                
                //             }
                //         )
                //     )
                // }

                let display_name: String = directory_path.split("/").collect::<Vec<&str>>().last().unwrap().to_string();
                
                DirectoryInfo {
                    item_path: display_name,
                    item_metadata: query_item_path_metadata(password, directory_path),
                    sub_path_items: vec_info
                }

        }
    }
}