use crate::tag_memory::TagMemory;
use crate::tag_sensors::adxl363 as adxl;
use crate::tag_sensors::*;
use libstuhfl::gen2::*;
use libstuhfl::prelude::*;
use serial_test::serial;
use chrono::Local;
use std::error::Error;
use std::fs::OpenOptions;
use std::io;
use std::io::Write;
type TestResult = Result<(), libstuhfl::error::Error>;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use ctrlc;


struct TemperatureData {
    timestamp: String,
    temperature: f32,
}
 
 //temp_log uses ctrl+c to stop
#[test]
#[serial]
fn temp_log() -> Result<(), Box<dyn Error>> {
    println!("Executing temp_log function");
    //atomic boolean to signal when to exit the program
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    //setting a Ctrl+C handler
    ctrlc::set_handler(move||{r.store(false, Ordering::SeqCst);
}).expect("Error setting Ctrl+C handler");

    //connect to reader
    let reader = Reader::autoconnect()?;

    let gen2_cfg = Gen2Cfg::builder().build()?;

    let config = Gen2Cfg::builder().build().unwrap();

    let mut reader = reader.configure_gen2(&gen2_cfg)?;

    reader.tune(TuningAlgorithm::Exact)?;

    let mut csv_filename = String::new();


    loop{
        if !running.load(Ordering::SeqCst){
            break;
        }

        //getting the current date
        let now = Local::now();
        csv_filename = format!("temperature_log {}.csv", now.format("%Y-%m-%d")).to_string();
        let now_str = now.format("%Y-%m-%d %H:%M:%S").to_string();
        let temp = get_sensor_data(&mut reader)?;
        let temperature_data = TemperatureData {
            timestamp: now_str,
            temperature: temp,
        };

        let mut csv_file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&csv_filename)?;

        if csv_file.metadata()?.len() == 0 {
            writeln!(&mut csv_file, "EPC, TID, Timestamp, Temperature (Celsius)")?;
        }

        let (_, tags) = reader.inventory_once()?;
        println!("num of tags: {}", tags.len());

        if tags.is_empty() {
            println!("No tag found");
            continue;
        }

        for tag in &tags {
           writeln!(
                &mut csv_file,
                "{}, {}, {}, {}",
                tag.epc,
                tag.tid,
                temperature_data.timestamp,
                temperature_data.temperature
           )?;
        } 
        std::thread::sleep(std::time::Duration::from_secs(5));

    }

    Ok(())
}

// new function that takes a specific epc for temp_log
fn specific_temp_epc(reader: &mut Gen2Reader, epc_to_find: HexID) -> Result<(), Box<dyn Error>>{
    println!("Executing temp_log function for individual EPC number");

     //atomic boolean to signal when to exit the program
    let running = Arc::new(AtomicBool::new(true));
     let r = running.clone();
     //setting a Ctrl+C handler
    ctrlc::set_handler(move||{r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl+C handler");
        reader.tune(TuningAlgorithm::Exact)?;
    
        let mut csv_filename = String::new();

        //set to repeat 5 times right now, can change to loop
        //loop
        for _ in 0..5 {
            if !running.load(Ordering::SeqCst){
                break;
            }
            //getting time and naming csv file
            let now = Local::now();
            csv_filename = format!("temperature_log {}.csv", now.format("%Y-%m-%d")).to_string();
            let now_str = now.format("%Y-%m-%d %H:%M:%S").to_string();
            let temp = get_sensor_data(reader)?;
            
            let mut csv_file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&csv_filename)?;
        
            //creating header
            if csv_file.metadata()?.len() == 0 {
                writeln!(&mut csv_file, "EPC, TID, Timestamp, Temperature (Celsius)")?;
            }

            //find epc number within tags
            let (_, tags) = reader.inventory_once()?;
            let mut found_tag = None;
            for tag in &tags {
                if tag.epc == epc_to_find{
                    found_tag = Some(tag);
                    break;
                }
            }
            //if it is found, create a temp structure and print info into csv
            if let Some(tag) = found_tag{
                reader.select(&tag.epc)?;
                let temperature_data = TemperatureData {
                    timestamp: now_str,
                    temperature: temp,
                };    
                writeln!(
                    &mut csv_file,
                    "{}, {}, {}, {}",
                    tag.epc,
                    tag.tid,
                    temperature_data.timestamp,
                    temperature_data.temperature
            )?;

            }
            std::thread::sleep(std::time::Duration::from_secs(5));

    }
    Ok(())

}

//new funtion that takes a specific epc for em_sensor_test
fn specific_sensor_test(reader: &mut Gen2Reader, epc_to_find: HexID) -> Result<(), Box<dyn Error>> {
    println!("Executing em_sensor_test for individual EPC number");    
    println!("checking...");
    reader.select(&epc_to_find)?;
    let temp = get_sensor_data(reader)?;
    println!("Got temp: {temp} °C");

    Ok(())
        
}

//error with original function
//new function that takes a specific epc for em_write_config
fn specific_write_config(reader: &mut Gen2Reader, epc_to_find: HexID) -> Result<(), Box<dyn Error>> {
    println!("Executing em_write_config for individual EPC number");
    println!("checking..");
    reader.select(&epc_to_find)?;
    const TEMP_SENSOR_CONTROL_WORD_1: u32 = 0xEC;
    const TEMP_SENSOR_CONTROL_WORD_2: u32 = 0xED;
    const TEMP_SENSOR_CONTROL_WORD_3: u32 = 0xEE;
    const IO_CONTROL_WORD: u32 = 0xF0;
    const BATTERY_MANAGEMENT_WORD_1: u32 = 0xF1;
    const BATTERY_MANAGEMENT_WORD_2: u32 = 0xF2;
    const TOTAL_WORD: u32 = 0xF3;
    const BAP_MODE_WORD: u32 = 0x10D;

    println!("Seting up tag...");

    println!("Writing temp sensor control words");

    reader.write(
        MemoryBank::User,
        TEMP_SENSOR_CONTROL_WORD_1,
        [0x00, 0x00],
        None,
    )?;

    reader.write(
        MemoryBank::User,
        TEMP_SENSOR_CONTROL_WORD_2,
        [0x00, 0x00],
        None,
    )?;

    reader.write(
        MemoryBank::User,
        TEMP_SENSOR_CONTROL_WORD_3,
        [0x00, 0x00],
        None,
    )?;

    println!("Writing IO control word");

    reader.write(MemoryBank::User, IO_CONTROL_WORD, [0x06, 0x00], None)?;

    println!("Writing Battery Management control word");

    reader.write(
        MemoryBank::User,
        BATTERY_MANAGEMENT_WORD_1,
        [0x00, 0x00],
        None,
    )?;

    reader.write(
        MemoryBank::User,
        BATTERY_MANAGEMENT_WORD_2,
        [0x00, 0x01],
        None,
    )?;

    println!("Writing TOTAL word");

    reader.write(MemoryBank::User, TOTAL_WORD, [0x00, 0x00], None)?;

    println!("Writing BAP mode word");

    reader.write(MemoryBank::User, BAP_MODE_WORD, [0x00, 0x00], None)?;
        


    Ok(())
}

//error with original function
//new function that takes a specific epc for em_bap_mode
fn specific_bap_mode(reader: &mut Gen2Reader, epc_to_find: HexID) -> Result<(), Box<dyn Error>>{
    println!("Executing em_bap_mode for individual EPC number");

    reader.select(&epc_to_find)?;
    const TEMP_SENSOR_CONTROL_WORD_1: u32 = 0xEC;
    const TEMP_SENSOR_CONTROL_WORD_2: u32 = 0xED;
    const TEMP_SENSOR_CONTROL_WORD_3: u32 = 0xEE;
    const IO_CONTROL_WORD: u32 = 0xF0;
    const BATTERY_MANAGEMENT_WORD_1: u32 = 0xF1;
    const BATTERY_MANAGEMENT_WORD_2: u32 = 0xF2;
    const TOTAL_WORD: u32 = 0xF3;
    const BAP_MODE_WORD: u32 = 0x10D;

    println!("Seting up tag...");

    println!("Writing temp sensor control words");

    reader.write(
        MemoryBank::User,
        TEMP_SENSOR_CONTROL_WORD_1,
        [0x00, 0x00],
        None,
    )?;

    reader.write(
        MemoryBank::User,
        TEMP_SENSOR_CONTROL_WORD_2,
        [0x00, 0x00],
        None,
    )?;

    reader.write(
        MemoryBank::User,
        TEMP_SENSOR_CONTROL_WORD_3,
        [0x00, 0x00],
        None,
    )?;

    println!("Writing IO control word");

    reader.write(MemoryBank::User, IO_CONTROL_WORD, [0xE0, 0x00], None)?;

    println!("Writing Battery Management control word");

    reader.write(
        MemoryBank::User,
        BATTERY_MANAGEMENT_WORD_1,
        [0x20, 0x01],
        None,
    )?;

    reader.write(
        MemoryBank::User,
        BATTERY_MANAGEMENT_WORD_2,
        [0x00, 0x01],
        None,
    )?;

    println!("Writing TOTAL word");

    reader.write(MemoryBank::User, TOTAL_WORD, [0x00, 0x00], None)?;

    println!("Writing BAP mode word");

    reader.write(MemoryBank::User, BAP_MODE_WORD, [0x00, 0x01], None)?;
   
    Ok(())
}

//error with original function
//new function that takes a specific epc for em_passive_mode
fn specific_passive_mode(reader: &mut Gen2Reader, epc_to_find: HexID) -> Result<(), Box<dyn Error>>{
    println!("Executing em_passive_mode for individual EPC number");
    reader.select(&epc_to_find)?;

    const TEMP_SENSOR_CONTROL_WORD_1: u32 = 0xEC;
    const TEMP_SENSOR_CONTROL_WORD_2: u32 = 0xED;
    const TEMP_SENSOR_CONTROL_WORD_3: u32 = 0xEE;
    const IO_CONTROL_WORD: u32 = 0xF0;
    const BATTERY_MANAGEMENT_WORD_1: u32 = 0xF1;
    const BATTERY_MANAGEMENT_WORD_2: u32 = 0xF2;
    const TOTAL_WORD: u32 = 0xF3;
    const BAP_MODE_WORD: u32 = 0x10D;

    println!("Seting up tag...");

    println!("Writing temp sensor control words");

    reader.write(
        MemoryBank::User,
        TEMP_SENSOR_CONTROL_WORD_1,
        [0x00, 0x00],
        None,
    )?;

    reader.write(
        MemoryBank::User,
        TEMP_SENSOR_CONTROL_WORD_2,
        [0x00, 0x00],
        None,
    )?;

    reader.write(
        MemoryBank::User,
        TEMP_SENSOR_CONTROL_WORD_3,
        [0x00, 0x00],
        None,
    )?;

    println!("Writing IO control word");

    reader.write(MemoryBank::User, IO_CONTROL_WORD, [0xE6, 0x00], None)?;

    println!("Writing Battery Management control word");

    reader.write(
        MemoryBank::User,
        BATTERY_MANAGEMENT_WORD_1,
        [0x00, 0x00],
        None,
    )?;

    reader.write(
        MemoryBank::User,
        BATTERY_MANAGEMENT_WORD_2,
        [0x00, 0x01],
        None,
    )?;

    println!("Writing TOTAL word");

    reader.write(MemoryBank::User, TOTAL_WORD, [0x00, 0x00], None)?;

    println!("Writing BAP mode word");

    reader.write(MemoryBank::User, BAP_MODE_WORD, [0x00, 0x00], None)?;

    println!("Rewriting Battery Management Word 2");

    // disable BAP control after writing BAP mode word
    reader.write(
        MemoryBank::User,
        BATTERY_MANAGEMENT_WORD_2,
        [0x00, 0x00],
        None,
    )?;
    
    Ok(())
}

//new function that takes a specific epc for em_read_config
fn specific_read_config(reader: &mut Gen2Reader, epc_to_find: HexID) -> Result<(), Box<dyn Error>> {
    println!("Executing em_read_config for individual EPC number");

    reader.select(&epc_to_find)?;

    const TEMP_SENSOR_CONTROL_WORD: u32 = 0xEC;
    const IO_CONTROL_WORD: u32 = 0xF0;
    const BATTERY_MANAGEMENT_WORD: u32 = 0xF1;
    const TOTAL_WORD: u32 = 0xF3;
    const BAP_MODE_WORD: u32 = 0x10D;

    println!("Reading Tag Settings...");

    let bytes = reader.read_alt(MemoryBank::User, TEMP_SENSOR_CONTROL_WORD, 3, None)?;
    println!("Temp Sensor Control Words: {bytes:02X?}");

    let bytes = reader.read_alt(MemoryBank::User, IO_CONTROL_WORD, 1, None)?;
    println!("I/O Control Word: {bytes:02X?}");

    let bytes = reader.read_alt(MemoryBank::User, BATTERY_MANAGEMENT_WORD, 2, None)?;
    println!("Battery Management Words: {bytes:02X?}");

    let bytes = reader.read_alt(MemoryBank::User, TOTAL_WORD, 1, None)?;
    println!("TOTAL Words: {bytes:02X?}");

    let bytes = reader.read_alt(MemoryBank::User, BAP_MODE_WORD, 1, None)?;
    println!("BAP Mode Word: {bytes:02X?}");
    
    Ok(())
}

//new function that takes a specific epc for em_verify_calibration
fn specific_verify_calibration(reader: &mut Gen2Reader, epc_to_find: HexID) -> Result<(), Box<dyn Error>> {
    reader.select(&epc_to_find)?;

    println!("Reading temperature sensor calibration words");

    let co = reader.read_alt(MemoryBank::Tid, 0x0D, 1, None)?;
    let co_int = u16::from_be_bytes([co[0], co[1]]);

    let cc = reader.read_alt(MemoryBank::User, 0xEF, 1, None)?;
    let cc_int = u16::from_be_bytes([cc[0], cc[1]]);

    // highest 11 bits must be identical
    if co_int & 0xFFE0 != cc_int & 0xFFE0 {
        println!("Error in temperature sensor calibration. Resetting...");
        reader.write(MemoryBank::User, 0xEF, [co[0], co[1]], None)?;
        println!("Done.")
    } else {
        println!("Temperature sensor calibration successfully verified.");
        let offset = if cc_int & 0b0000_0000_0001_0000 != 0 {
            cc_int | 0b1111_1111_1110_0000 // sign extension negative
        } else {
            cc_int & 0b0000_0000_0001_1111 // sign extension positive
        };

        println!("Fine trim value: {} °C", process_temp(offset));
          
    }
    Ok(())
}

//new function for purple tags that takes a specific epc for adxl_setup_config
fn specific_adxl_setup(reader: &mut Gen2Reader, epc_to_find: HexID) -> Result<(), Box<dyn Error>> {
    reader.select(&epc_to_find)?;

    assert!(adxl::test_adxl_connection(reader)?);

    println!("Setting up tag...");
    Ok(adxl::setup(reader)?)
}

//new function for purple tags that takes a specific epc for adxl_sensor_test
fn specific_adxl_sensor(reader: &mut Gen2Reader, epc_to_find: HexID) -> Result<(), Box<dyn Error>> {
    reader.select(&epc_to_find)?;

    println!("Charging up semi-BAP");
    reader.inventory(2000, Box::new(|_| {}))?;
    reader.inventory_once()?;

    println!("Checking ADXL connection...");

    assert!(adxl::test_adxl_connection(reader)?);

    println!("Configuring ADXL...");

    adxl::setup(reader)?;

    println!("Turning on measurements...");

    adxl::turn_on(reader)?;

    // wait for 3 measurements to be taken
    let stime = std::time::Instant::now();
    let duration = std::time::Duration::from_secs_f32(3.0 / 12.5);

    while stime.elapsed() < duration {
        reader.inventory(20, Box::new(|_| {}))?;
    }
    reader.inventory_once()?;

    println!("Turning off measurements...");

    adxl::turn_off(reader)?;

    let num_samples = adxl::get_num_fifo_entries(reader)?;

    println!("Got {num_samples} samples (discarding 3)");

    assert!(num_samples >= 3);

    println!("Reading {} measurements", (num_samples - 3) / 3);

    // discard first 3 samples in fifo
    for _ in 0..3 {
        adxl::read_fifo(reader, 2)?;
    }

    // Read all the measurements we got
    let samples = adxl::get_fifo_entries(reader)?;
    for (i, sample) in samples.iter().enumerate() {
        println!("Sample {i}: {sample}")
    }

    Ok(())
}

//new function for purple tags that takes a specific epc for improved_vibration
fn specific_improved_vibration(reader: &mut Gen2Reader, epc_to_find: HexID) -> Result<(), Box<dyn Error>> {
    reader.select(&epc_to_find)?;

    // Prepare test
    println!("Charging up semi-BAP");
    let stats = reader.inventory(2000, Box::new(|_| {}))?;
    reader.inventory_once()?;
    let mut peak_rssi = stats.rssi_log_mean;

    println!("Checking ADXL connection...");
    assert!(adxl::test_adxl_connection(reader)?);

    println!("Configuring ADXL...");
    adxl::setup(reader)?;

    // Actual Reading
    println!("Turning on measurements...");
    adxl::turn_on(reader)?;

    // wait for 3 measurements to be taken
    let stime_chrono = chrono::Utc::now();
    let stime = std::time::Instant::now();
    let duration = std::time::Duration::from_secs_f32(3.0 / 12.5);
    while stime.elapsed() < duration {
        let stats = reader.inventory(20, Box::new(|_| {}))?;
        peak_rssi = std::cmp::max(stats.rssi_log_mean, peak_rssi);
    }
    // reset Gen2 errors in firmware
    reader.inventory_once()?;

    // End reading
    println!("Turning off measurements...");
    adxl::turn_off(reader)?;

    let num_samples = adxl::get_num_fifo_entries(reader)?;

    // the first 3 samples are invalid data created by the setup process
    println!("Got {num_samples} samples (discarding 3)");
    assert!(num_samples >= 3);
    for _ in 0..3 {
        adxl::read_fifo(reader, 2)?;
    }

    let reflected_i;
    let reflected_q;

    // Reflected power not implemented in the wrapper
    unsafe {
        use libstuhfl_sys as ffi;

        let mut param = ffi::STUHFL_T_ST25RU3993_FreqReflectedPowerInfo {
            frequency: 865000,
            applyTunerSetting: true,
            reflectedI: 0,
            reflectedQ: 0,
        };

        let ret_code = ffi::Get_FreqReflectedPower(&mut param);
        assert!(ret_code == 0); // error handling is normally contained in the wrapper

        reflected_i = param.reflectedI;
        reflected_q = param.reflectedQ;
    }

    println!("Peak RSSI: {peak_rssi}, Reflected Power: {reflected_i} (i) {reflected_q} (q)");

    assert!(num_samples % 3 == 0);
    println!("Reading {} measurements", (num_samples - 3) / 3);

    // Read all the measurements we got
    let samples = adxl::get_fifo_entries(reader)?;

    let interval = (1000.0 / 12.5) as i64;

    for i in 0..(num_samples as usize/ 3) {
        let index = i*3;
        if index+2 < samples.len(){
        println!(
            "[{}] {} {} {}",
            stime_chrono + chrono::Duration::milliseconds(interval * (i as i64 + 1)),
            samples[i * 3],
            samples[i * 3 + 1],
            samples[i * 3 + 2]
        );
        }else{
            println!("error");
            break;
        }
        
    }

    for (i, sample) in samples.iter().enumerate() {
        println!("Sample {i}: {sample}");
    }

    Ok(())
}

#[test]
#[serial]
fn select_tag() ->  Result<(), Box<dyn Error>> {
    let reader = Reader::autoconnect()?;
    let config = Gen2Cfg::builder().build().unwrap();
    let mut reader = reader.configure_gen2(&config)?;
    reader.tune(TuningAlgorithm::Exact)?;


    let (_, tags) = reader.inventory_once()?;


    if tags.is_empty() {
        panic!("No tag found");
    }
    


    println!("Found the following tags: ");
    for i in 0..tags.len() {
        println!("{}: EPC: {}, TID: {}", i, tags[i].epc, tags[i].tid);
    }


    // Create mutable String to store user input
    let mut input = String::new();


    // Prompt the user for input
    println!("Pick tag EPC number to select: ");


    // Read user input into the String
    io::stdin().read_line(&mut input).expect("Failed to read input");
    println!("You picked: {}", input);
    let input_hex = input.trim();
    

    //check to see if user input is a valid EPC num
    let mut selected_tag: Option<&InventoryTag> = None;
    for tag in &tags{
        let tag_epc_string = format!("{}", tag.epc);
        if tag_epc_string == input_hex{
            selected_tag = Some(tag);
            break;
        }
    }

    if let Some(tag) = selected_tag{
        println!("You successfully picked {}", tag.epc);
        reader.select(&tag.epc)?;
        //let tag_epc_string = format!("{}", tag.epc);
        let epc_to_find = tag.epc.clone();

        
        //let user pick function to perform
        println!("Select the function you would like to perform: 
            1: temp_log function
            2: em_sensor_test
            3: Error: em_write_config
            4: Error: em_bap_mode
            5: Error: em_passive_mode
            6: em_read_config
            7: em_verify_calibration
            8: Purple Tag - adxl_setup_config
            9: Purple Tag - adxl_sensor_test
            10: Purple Tag - improved_vibration");
        //string for user to choose which function to test
        let mut choose_test = String::new();
        //read user input
        io::stdin().read_line(&mut choose_test).expect("Failed to read input");
        //check if user input is valid
        match choose_test.trim(){
             "1" => match specific_temp_epc(&mut reader, epc_to_find){
                Ok(()) => {
                    println!("Temperature Log completed successfully");
                },
                Err(err) =>{
                    println!("Error occured: {}", err);
                    //fails test explicitly
                    assert!(false, "Error occurred: {}", err);
                }
                
            }
             "2" => match specific_sensor_test(&mut reader, epc_to_find){
            Ok(()) => {
                println!("em_sensor test completed successfully");
            },
            Err(err) =>{
                println!("Error occured: {}", err);
                //fails test explicitly
                assert!(false, "Error occurred: {}", err);
            }
            }
            "3" => match specific_write_config(&mut reader, epc_to_find){
                Ok(()) => {
                    println!("em_write_config test completed successfully");
                },
                Err(err) =>{
                    println!("Error occured: {}", err);
                    //fails test explicitly
                    assert!(false, "Error occurred: {}", err);
                }
                }
            "4" => match specific_bap_mode(&mut reader, epc_to_find){
                Ok(()) => {
                    println!("em_bap_mode test completed successfully");
                },
                Err(err) =>{
                    println!("Error occured: {}", err);
                    //fails test explicitly
                    assert!(false, "Error occurred: {}", err);
                }
                }
            "5" => match specific_passive_mode(&mut reader, epc_to_find){
                Ok(()) => {
                    println!("em_passive_mode test completed successfully");
                },
                Err(err) =>{
                    println!("Error occured: {}", err);
                    //fails test explicitly
                    assert!(false, "Error occurred: {}", err);
                }
                }
            "6" => match specific_read_config(&mut reader, epc_to_find){
                Ok(()) => {
                    println!("em_read_config test completed successfully");
                },
                Err(err) =>{
                    println!("Error occured: {}", err);
                    //fails test explicitly
                    assert!(false, "Error occurred: {}", err);
                }
                }
            "7" => match specific_verify_calibration(&mut reader, epc_to_find){
                    Ok(()) => {
                        println!("em_verify_calibration test completed successfully");
                    },
                    Err(err) =>{
                        println!("Error occured: {}", err);
                        //fails test explicitly
                        assert!(false, "Error occurred: {}", err);
                    }
                    }   
            "8" => match specific_adxl_setup(&mut reader, epc_to_find){
                Ok(()) => {
                    println!("adxl_setup_config test completed successfully");
                },
                Err(err) =>{
                    println!("Error occured: {}", err);
                    //fails test explicitly
                    assert!(false, "Error occurred: {}", err);
                }
                } 
            "9" => match specific_adxl_sensor(&mut reader, epc_to_find){
                Ok(()) => {
                    println!("adxl_sensor_test completed successfully");
                },
                Err(err) =>{
                    println!("Error occured: {}", err);
                    //fails test explicitly
                    assert!(false, "Error occurred: {}", err);
                }
                }  
            "10" => match specific_improved_vibration(&mut reader, epc_to_find){
                Ok(()) => {
                    println!("improved_vibration completed successfully");
                },
                Err(err) =>{
                    println!("Error occured: {}", err);
                    //fails test explicitly
                    assert!(false, "Error occurred: {}", err);
                }
                }    
            _ => println!("Invalid input. Please enter one of the number choices."),
        }

    } else {
       println! ("Error: Invalid tag EPC number");
    }
    Ok(())
}


//make a number to hex function
fn numbers_to_hex(numbers_str: String) -> Option<String> {
    //parse into unsigned 64 bit integer
    let number: u64 = match numbers_str.parse(){
        Ok(n) => n,
        Err(_) => {
            return None; //return none if the parsing fails
    
        }
    };
    let hex_str = format!("{:X}", number);
    Some(hex_str) //return the hexid string wrapped in Some
}

#[test]
fn process_temp_test() {
    assert_eq!(process_temp(0b1_0000_0001), -63.75);
    assert_eq!(process_temp(0b0_0000_0000), 0.00);
    assert_eq!(process_temp(0b0_1111_1111), 63.75);
}

#[test]
#[serial]
fn find_tags() -> TestResult {
    let reader = Reader::autoconnect()?;

    let config = Gen2Cfg::builder().build().unwrap();
  let mut reader = reader.configure_gen2(&config)?;

    reader.tune(TuningAlgorithm::Exact)?;

    println!("Inventorying tags...");

    let (_, tags) = reader.inventory_once()?;

    for tag in tags {
        println!("EPC: {}, TID: {}", tag.epc, tag.tid);
    }
  

    Ok(())
}

#[test]
#[serial]
fn em_write_config() -> TestResult {
    let reader = Reader::autoconnect()?;

    let config = Gen2Cfg::builder().build().unwrap();

    let mut reader = reader.configure_gen2(&config)?;

    reader.tune(TuningAlgorithm::Exact)?;

    let (_, tags) = reader.inventory_once()?;

    if tags.is_empty() {
        panic!("No tag found")
    }

    reader.select(&tags[0].epc)?;

    const TEMP_SENSOR_CONTROL_WORD_1: u32 = 0xEC;
    const TEMP_SENSOR_CONTROL_WORD_2: u32 = 0xED;
    const TEMP_SENSOR_CONTROL_WORD_3: u32 = 0xEE;
    const IO_CONTROL_WORD: u32 = 0xF0;
    const BATTERY_MANAGEMENT_WORD_1: u32 = 0xF1;
    const BATTERY_MANAGEMENT_WORD_2: u32 = 0xF2;
    const TOTAL_WORD: u32 = 0xF3;
    const BAP_MODE_WORD: u32 = 0x10D;

    println!("Seting up tag...");

    println!("Writing temp sensor control words");

    reader.write(
        MemoryBank::User,
        TEMP_SENSOR_CONTROL_WORD_1,
        [0x00, 0x00],
        None,
    )?;

    reader.write(
        MemoryBank::User,
        TEMP_SENSOR_CONTROL_WORD_2,
        [0x00, 0x00],
        None,
    )?;

    reader.write(
        MemoryBank::User,
        TEMP_SENSOR_CONTROL_WORD_3,
        [0x00, 0x00],
        None,
    )?;

    println!("Writing IO control word");

    reader.write(MemoryBank::User, IO_CONTROL_WORD, [0x06, 0x00], None)?;

    println!("Writing Battery Management control word");

    reader.write(
        MemoryBank::User,
        BATTERY_MANAGEMENT_WORD_1,
        [0x00, 0x00],
        None,
    )?;

    reader.write(
        MemoryBank::User,
        BATTERY_MANAGEMENT_WORD_2,
        [0x00, 0x01],
        None,
    )?;

    println!("Writing TOTAL word");

    reader.write(MemoryBank::User, TOTAL_WORD, [0x00, 0x00], None)?;

    println!("Writing BAP mode word");

    reader.write(MemoryBank::User, BAP_MODE_WORD, [0x00, 0x00], None)?;

    Ok(())
}

#[test]
#[serial]
fn em_sensor_test() -> TestResult{
    let reader = Reader::autoconnect()?;

    let config = Gen2Cfg::builder().build().unwrap();

    let mut reader = reader.configure_gen2(&config)?;

    reader.tune(TuningAlgorithm::Exact)?;

    let (_, tags) = reader.inventory_once()?;

    if tags.is_empty() {
        panic!("No tag found")
    }

    reader.select(&tags[0].epc)?;

    let temp = get_sensor_data(&mut reader)?;

    println!("Got temp: {temp} °C");

    Ok(())
}

#[test]
#[serial]
fn em_bap_mode() -> TestResult {
    let reader = Reader::autoconnect()?;

    let config = Gen2Cfg::builder().build().unwrap();

    let mut reader = reader.configure_gen2(&config)?;

    reader.tune(TuningAlgorithm::Exact)?;

    let (_, tags) = reader.inventory_once()?;

    if tags.is_empty() {
        panic!("No tag found")
    }

    reader.select(&tags[0].epc)?;

    const TEMP_SENSOR_CONTROL_WORD_1: u32 = 0xEC;
    const TEMP_SENSOR_CONTROL_WORD_2: u32 = 0xED;
    const TEMP_SENSOR_CONTROL_WORD_3: u32 = 0xEE;
    const IO_CONTROL_WORD: u32 = 0xF0;
    const BATTERY_MANAGEMENT_WORD_1: u32 = 0xF1;
    const BATTERY_MANAGEMENT_WORD_2: u32 = 0xF2;
    const TOTAL_WORD: u32 = 0xF3;
    const BAP_MODE_WORD: u32 = 0x10D;

    println!("Seting up tag...");

    println!("Writing temp sensor control words");

    reader.write(
        MemoryBank::User,
        TEMP_SENSOR_CONTROL_WORD_1,
        [0x00, 0x00],
        None,
    )?;

    reader.write(
        MemoryBank::User,
        TEMP_SENSOR_CONTROL_WORD_2,
        [0x00, 0x00],
        None,
    )?;

    reader.write(
        MemoryBank::User,
        TEMP_SENSOR_CONTROL_WORD_3,
        [0x00, 0x00],
        None,
    )?;

    println!("Writing IO control word");

    reader.write(MemoryBank::User, IO_CONTROL_WORD, [0xE0, 0x00], None)?;

    println!("Writing Battery Management control word");

    reader.write(
        MemoryBank::User,
        BATTERY_MANAGEMENT_WORD_1,
        [0x20, 0x01],
        None,
    )?;

    reader.write(
        MemoryBank::User,
        BATTERY_MANAGEMENT_WORD_2,
        [0x00, 0x01],
        None,
    )?;

    println!("Writing TOTAL word");

    reader.write(MemoryBank::User, TOTAL_WORD, [0x00, 0x00], None)?;

    println!("Writing BAP mode word");

    reader.write(MemoryBank::User, BAP_MODE_WORD, [0x00, 0x01], None)?;

    Ok(())
}

#[test]
#[serial]
fn em_passive_mode() -> TestResult {
    let reader = Reader::autoconnect()?;

    let config = Gen2Cfg::builder().build().unwrap();

    let mut reader = reader.configure_gen2(&config)?;

    reader.tune(TuningAlgorithm::Exact)?;

    let (_, tags) = reader.inventory_once()?;

    if tags.is_empty() {
        panic!("No tag found")
    }

    reader.select(&tags[0].epc)?;

    const TEMP_SENSOR_CONTROL_WORD_1: u32 = 0xEC;
    const TEMP_SENSOR_CONTROL_WORD_2: u32 = 0xED;
    const TEMP_SENSOR_CONTROL_WORD_3: u32 = 0xEE;
    const IO_CONTROL_WORD: u32 = 0xF0;
    const BATTERY_MANAGEMENT_WORD_1: u32 = 0xF1;
    const BATTERY_MANAGEMENT_WORD_2: u32 = 0xF2;
    const TOTAL_WORD: u32 = 0xF3;
    const BAP_MODE_WORD: u32 = 0x10D;

    println!("Seting up tag...");

    println!("Writing temp sensor control words");

    reader.write(
        MemoryBank::User,
        TEMP_SENSOR_CONTROL_WORD_1,
        [0x00, 0x00],
        None,
    )?;

    reader.write(
        MemoryBank::User,
        TEMP_SENSOR_CONTROL_WORD_2,
        [0x00, 0x00],
        None,
    )?;

    reader.write(
        MemoryBank::User,
        TEMP_SENSOR_CONTROL_WORD_3,
        [0x00, 0x00],
        None,
    )?;

    println!("Writing IO control word");

    reader.write(MemoryBank::User, IO_CONTROL_WORD, [0xE6, 0x00], None)?;

    println!("Writing Battery Management control word");

    reader.write(
        MemoryBank::User,
        BATTERY_MANAGEMENT_WORD_1,
        [0x00, 0x00],
        None,
    )?;

    reader.write(
        MemoryBank::User,
        BATTERY_MANAGEMENT_WORD_2,
        [0x00, 0x01],
        None,
    )?;

    println!("Writing TOTAL word");

    reader.write(MemoryBank::User, TOTAL_WORD, [0x00, 0x00], None)?;

    println!("Writing BAP mode word");

    reader.write(MemoryBank::User, BAP_MODE_WORD, [0x00, 0x00], None)?;

    println!("Rewriting Battery Management Word 2");

    // disable BAP control after writing BAP mode word
    reader.write(
        MemoryBank::User,
        BATTERY_MANAGEMENT_WORD_2,
        [0x00, 0x00],
        None,
    )?;

    Ok(())
}

#[test]
#[serial]
fn em_read_config() -> TestResult {
    let reader = Reader::autoconnect()?;

    let config = Gen2Cfg::builder().build().unwrap();

    let mut reader = reader.configure_gen2(&config)?;

    reader.tune(TuningAlgorithm::Exact)?;

    let (_, tags) = reader.inventory_once()?;

    if tags.is_empty() {
        panic!("No tag found")
    }

    reader.select(&tags[0].epc)?;

    const TEMP_SENSOR_CONTROL_WORD: u32 = 0xEC;
    const IO_CONTROL_WORD: u32 = 0xF0;
    const BATTERY_MANAGEMENT_WORD: u32 = 0xF1;
    const TOTAL_WORD: u32 = 0xF3;
    const BAP_MODE_WORD: u32 = 0x10D;

    println!("Reading Tag Settings...");

    let bytes = reader.read_alt(MemoryBank::User, TEMP_SENSOR_CONTROL_WORD, 3, None)?;
    println!("Temp Sensor Control Words: {bytes:02X?}");

    let bytes = reader.read_alt(MemoryBank::User, IO_CONTROL_WORD, 1, None)?;
    println!("I/O Control Word: {bytes:02X?}");

    let bytes = reader.read_alt(MemoryBank::User, BATTERY_MANAGEMENT_WORD, 2, None)?;
    println!("Battery Management Words: {bytes:02X?}");

    let bytes = reader.read_alt(MemoryBank::User, TOTAL_WORD, 1, None)?;
    println!("TOTAL Words: {bytes:02X?}");

    let bytes = reader.read_alt(MemoryBank::User, BAP_MODE_WORD, 1, None)?;
    println!("BAP Mode Word: {bytes:02X?}");

    Ok(())
}

#[test]
#[serial]
fn em_verify_calibration() -> TestResult {
    let reader = Reader::autoconnect()?;

    let config = Gen2Cfg::builder().build().unwrap();

    let mut reader = reader.configure_gen2(&config)?;

    reader.tune(TuningAlgorithm::Exact)?;

    let (_, tags) = reader.inventory_once()?;

    if tags.is_empty() {
        panic!("No tag found")
    }

    reader.select(&tags[0].epc)?;

    println!("Reading temperature sensor calibration words");

    let co = reader.read_alt(MemoryBank::Tid, 0x0D, 1, None)?;
    let co_int = u16::from_be_bytes([co[0], co[1]]);

    let cc = reader.read_alt(MemoryBank::User, 0xEF, 1, None)?;
    let cc_int = u16::from_be_bytes([cc[0], cc[1]]);

    // highest 11 bits must be identical
    if co_int & 0xFFE0 != cc_int & 0xFFE0 {
        println!("Error in temperature sensor calibration. Resetting...");
        reader.write(MemoryBank::User, 0xEF, [co[0], co[1]], None)?;
        println!("Done.")
    } else {
        println!("Temperature sensor calibration successfully verified.");
        let offset = if cc_int & 0b0000_0000_0001_0000 != 0 {
            cc_int | 0b1111_1111_1110_0000 // sign extension negative
        } else {
            cc_int & 0b0000_0000_0001_1111 // sign extension positive
        };

        println!("Fine trim value: {} °C", process_temp(offset));
    }

    Ok(())
}

#[test]
#[serial]
fn adxl_read_test() -> TestResult {
    let reader = Reader::autoconnect()?;

    let config = Gen2Cfg::builder().build().unwrap();

    let mut reader = reader.configure_gen2(&config)?;

    reader.tune(TuningAlgorithm::Exact)?;

    let (_, tags) = reader.inventory_once()?;

    if tags.is_empty() {
        panic!("No tag found")
    }

    reader.select(&tags[0].epc)?;

    println!("Reading ID Registers");
    let bytes = adxl::read_register(&mut reader, adxl::Register::DevIdAd, 3)?;

    println!("Got IDs: {:02X?}", &bytes);

    assert_eq!(bytes, &[0xAD, 0x1D, 0xF3]);

    println!("IDs are correct.");

    Ok(())
}

#[test]
#[serial]
fn adxl_write_test() -> TestResult {
    let reader = Reader::autoconnect()?;

    let config = Gen2Cfg::builder().build().unwrap();

    let mut reader = reader.configure_gen2(&config)?;

    reader.tune(TuningAlgorithm::Exact)?;

    let (_, tags) = reader.inventory_once()?;

    if tags.is_empty() {
        panic!("No tag found")
    }

    reader.select(&tags[0].epc)?;

    let data = &[0x12, 0x34];

    let address = adxl::Register::TimeInactL;

    let backup = adxl::read_register(&mut reader, address, data.len() as u16)?;

    println!("Writing Register");
    adxl::write_register(&mut reader, address, data)?;

    std::thread::sleep(std::time::Duration::from_millis(100));

    let new = adxl::read_register(&mut reader, address, data.len() as u16)?;

    assert_eq!(new, data);

    println!("Resetting Register");
    adxl::write_register(&mut reader, address, &backup)?;

    Ok(())
}

#[test]
#[serial]
fn adxl_setup_config() -> TestResult {
    let reader = Reader::autoconnect()?;

    let config = Gen2Cfg::builder().build().unwrap();

    let mut reader = reader.configure_gen2(&config)?;

    reader.tune(TuningAlgorithm::Exact)?;

    let (_, tags) = reader.inventory_once()?;

    if tags.is_empty() {
        panic!("No tag found")
    }

    reader.select(&tags[0].epc)?;

    assert!(adxl::test_adxl_connection(&mut reader)?);

    println!("Setting up tag...");
    adxl::setup(&mut reader)
}

#[test]
#[serial]
fn adxl_sensor_test() -> TestResult {
    let reader = Reader::autoconnect()?;

    let config = Gen2Cfg::builder().build().unwrap();

    let mut reader = reader.configure_gen2(&config)?;

    reader.tune(TuningAlgorithm::Exact)?;

    let (_, tags) = reader.inventory_once()?;

    if tags.is_empty() {
        panic!("No tag found")
    }

    reader.select(&tags[0].epc)?;

    println!("Charging up semi-BAP");
    reader.inventory(2000, Box::new(|_| {}))?;
    reader.inventory_once()?;

    println!("Checking ADXL connection...");

    assert!(adxl::test_adxl_connection(&mut reader)?);

    println!("Configuring ADXL...");

    adxl::setup(&mut reader)?;

    println!("Turning on measurements...");

    adxl::turn_on(&mut reader)?;

    // wait for 3 measurements to be taken
    let stime = std::time::Instant::now();
    let duration = std::time::Duration::from_secs_f32(3.0 / 12.5);

    while stime.elapsed() < duration {
        reader.inventory(20, Box::new(|_| {}))?;
    }
    reader.inventory_once()?;

    println!("Turning off measurements...");

    adxl::turn_off(&mut reader)?;

    let num_samples = adxl::get_num_fifo_entries(&mut reader)?;

    println!("Got {num_samples} samples (discarding 3)");

    assert!(num_samples >= 3);

    println!("Reading {} measurements", (num_samples - 3) / 3);

    // discard first 3 samples in fifo
    for _ in 0..3 {
        adxl::read_fifo(&mut reader, 2)?;
    }

    // Read all the measurements we got
    let samples = adxl::get_fifo_entries(&mut reader)?;
    for (i, sample) in samples.iter().enumerate() {
        println!("Sample {i}: {sample}")
    }

    Ok(())
}

#[test]
#[serial]
fn adxl_self_test() -> TestResult {
    let reader = Reader::autoconnect()?;

    let config = Gen2Cfg::builder().build().unwrap();

    let mut reader = reader.configure_gen2(&config)?;

    reader.tune(TuningAlgorithm::Exact)?;

    let (_, tags) = reader.inventory_once()?;

    if tags.is_empty() {
        panic!("No tag found")
    }

    reader.select(&tags[0].epc)?;

    assert!(adxl::test_adxl_connection(&mut reader)?);

    // Step 1: Read acceleration data for the x-, y-, and x-axes
    let data = adxl::read_register(&mut reader, adxl::Register::XDataL, 6)?;

    // Step 2: Assert self test by setting the ST bit in the SELF_TEST register
    adxl::write_register(&mut reader, adxl::Register::SelfTest, &[0x01])?;

    // Step 3: Wait 1/ODR for the output to settle to its new value
    std::thread::sleep(std::time::Duration::from_millis(120));

    // Step 4: Read acceleration data for the x-, y- and z-axes. Compare to the values from Step 1, and convert
    // the difference from LSB to mg by multiplying by the scale actor. If the observed difference falls within
    // the the self test output change specification listed in Table 2, the device passes self test and is
    // deemed operational.
    let new_data = adxl::read_register(&mut reader, adxl::Register::XDataL, 6)?;

    // TODO: COMPARE VALUES

    println!("Old value: {:02X?}", &data);
    println!("New value: {:02X?}", &new_data);

    // Step 5: Deassert self test by clearing hte ST bit in the SELF_TEST register
    adxl::write_register(&mut reader, adxl::Register::SelfTest, &[0x00])?;

    Ok(())
}

#[test]
#[serial]
fn improved_vibration() -> TestResult {
    // Initial setup
    let reader = Reader::autoconnect()?;
    let config = Gen2Cfg::builder().build().unwrap();
    let mut reader = reader.configure_gen2(&config)?;

    // Scanning for tags
    reader.tune(TuningAlgorithm::Exact)?;
    let (_, tags) = reader.inventory_once()?;
    if tags.is_empty() {
        panic!("No tag found")
    } else {
        let epc = &tags[0].epc;
        println!("Selecting tag with EPC {epc}");
        reader.select(epc)?;
    }

    // Prepare test
    println!("Charging up semi-BAP");
    let stats = reader.inventory(2000, Box::new(|_| {}))?;
    reader.inventory_once()?;
    let mut peak_rssi = stats.rssi_log_mean;

    println!("Checking ADXL connection...");
    assert!(adxl::test_adxl_connection(&mut reader)?);

    println!("Configuring ADXL...");
    adxl::setup(&mut reader)?;

    // Actual Reading
    println!("Turning on measurements...");
    adxl::turn_on(&mut reader)?;

    // wait for 3 measurements to be taken
    let stime_chrono = chrono::Utc::now();
    let stime = std::time::Instant::now();
    let duration = std::time::Duration::from_secs_f32(3.0 / 12.5);
    while stime.elapsed() < duration {
        let stats = reader.inventory(20, Box::new(|_| {}))?;
        peak_rssi = std::cmp::max(stats.rssi_log_mean, peak_rssi);
    }
    // reset Gen2 errors in firmware
    reader.inventory_once()?;

    // End reading
    println!("Turning off measurements...");
    adxl::turn_off(&mut reader)?;

    let num_samples = adxl::get_num_fifo_entries(&mut reader)?;

    // the first 3 samples are invalid data created by the setup process
    println!("Got {num_samples} samples (discarding 3)");
    assert!(num_samples >= 3);
    for _ in 0..3 {
        adxl::read_fifo(&mut reader, 2)?;
    }

    let reflected_i;
    let reflected_q;

    // Reflected power not implemented in the wrapper
    unsafe {
        use libstuhfl_sys as ffi;

        let mut param = ffi::STUHFL_T_ST25RU3993_FreqReflectedPowerInfo {
            frequency: 865000,
            applyTunerSetting: true,
            reflectedI: 0,
            reflectedQ: 0,
        };

        let ret_code = ffi::Get_FreqReflectedPower(&mut param);
        assert!(ret_code == 0); // error handling is normally contained in the wrapper

        reflected_i = param.reflectedI;
        reflected_q = param.reflectedQ;
    }

    println!("Peak RSSI: {peak_rssi}, Reflected Power: {reflected_i} (i) {reflected_q} (q)");

    assert!(num_samples % 3 == 0);
    println!("Reading {} measurements", (num_samples - 3) / 3);

    // Read all the measurements we got
    let samples = adxl::get_fifo_entries(&mut reader)?;

    let interval = (1000.0 / 12.5) as i64;

    for i in 0..num_samples as usize / 3 {
        println!(
            "[{}] {} {} {}",
            stime_chrono + chrono::Duration::milliseconds(interval * (i as i64 + 1)),
            samples[i * 3],
            samples[i * 3 + 1],
            samples[i * 3 + 2]
        );
    }
    println!("Length of samples: {}", samples.len());

    for (i, sample) in samples.iter().enumerate() {
        println!("Sample {i}: {sample}");
    }

    Ok(())
}

#[test]
#[serial]
fn em_pseudo_bap_mode() -> TestResult {
    let reader = Reader::autoconnect()?;

    let config = Gen2Cfg::builder().build().unwrap();

    let mut reader = reader.configure_gen2(&config)?;

    reader.tune(TuningAlgorithm::Exact)?;

    let (_, tags) = reader.inventory_once()?;

    if tags.is_empty() {
        panic!("No tag found")
    }

    reader.select(&tags[0].epc)?;

    const TEMP_SENSOR_CONTROL_WORD_1: u32 = 0xEC;
    const TEMP_SENSOR_CONTROL_WORD_2: u32 = 0xED;
    const TEMP_SENSOR_CONTROL_WORD_3: u32 = 0xEE;
    const IO_CONTROL_WORD: u32 = 0xF0;
    const BATTERY_MANAGEMENT_WORD_1: u32 = 0xF1;
    const BATTERY_MANAGEMENT_WORD_2: u32 = 0xF2;
    const TOTAL_WORD: u32 = 0xF3;
    const BAP_MODE_WORD: u32 = 0x10D;

    println!("Seting up tag...");

    println!("Writing temp sensor control words");

    reader.write(
        MemoryBank::User,
        TEMP_SENSOR_CONTROL_WORD_1,
        [0x00, 0x00],
        None,
    )?;

    reader.write(
        MemoryBank::User,
        TEMP_SENSOR_CONTROL_WORD_2,
        [0x00, 0x00],
        None,
    )?;

    reader.write(
        MemoryBank::User,
        TEMP_SENSOR_CONTROL_WORD_3,
        [0x00, 0x00],
        None,
    )?;

    println!("Writing IO control word");

    reader.write(MemoryBank::User, IO_CONTROL_WORD, [0x06, 0x00], None)?;

    println!("Writing Battery Management control word");

    reader.write(
        MemoryBank::User,
        BATTERY_MANAGEMENT_WORD_1,
        [0x00, 0x00],
        None,
    )?;

    reader.write(
        MemoryBank::User,
        BATTERY_MANAGEMENT_WORD_2,
        [0x00, 0x01], // allow BAP to be enabled on command
        None,
    )?;

    println!("Writing TOTAL word");

    reader.write(MemoryBank::User, TOTAL_WORD, [0x00, 0x00], None)?;

    println!("Writing BAP mode word");

    reader.write(MemoryBank::User, BAP_MODE_WORD, [0x00, 0x00], None)?;

    Ok(())
}

#[test]
#[serial]
fn em_pseudo_bap_test() -> TestResult {
    let reader = Reader::autoconnect()?;

    let config = Gen2Cfg::builder().build().unwrap();

    let mut reader = reader.configure_gen2(&config)?;

    reader.tune(TuningAlgorithm::Exact)?;

    let (_, tags) = reader.inventory_once()?;

    if tags.is_empty() {
        panic!("No tag found")
    }

    reader.select(&tags[0].epc)?;

    const SENSOR_DATA_MSW: u32 = 0x100;
    // const SENSOR_DATA_LSW: u32 = 0x101;
    const BAP_MODE_WORD: u32 = 0x10D;

    println!("Discharging capacitor...");

    reader.write(MemoryBank::User, BAP_MODE_WORD, [0x00, 0x01], None)?; // enable BAP

    let stime = std::time::Instant::now();
    let duration = std::time::Duration::from_secs_f32(0.5); // delay & inventory to power-cycle
    while stime.elapsed() < duration {}

    reader.inventory_once()?;

    let stime = std::time::Instant::now();
    let duration = std::time::Duration::from_secs_f32(20.0); // delay again to discharge
    while stime.elapsed() < duration {}

    println!("Charging capacitor...");

    while reader
        .write(MemoryBank::User, BAP_MODE_WORD, [0x00, 0x00], None)
        .is_err()
    {}

    reader.inventory(200, Box::new(|_| ())).ok();

    println!("Initiating a psuedo-bap measurement...");

    let stime = std::time::Instant::now();
    let duration = std::time::Duration::from_secs_f32(0.5); // delay & inventory to power-cycle
    while stime.elapsed() < duration {}
    reader.inventory_once()?;

    reader.write(MemoryBank::User, BAP_MODE_WORD, [0x00, 0x01], None)?;

    let stime = std::time::Instant::now();
    let duration = std::time::Duration::from_secs_f32(0.5); // delay & inventory to power-cycle
    while stime.elapsed() < duration {}
    reader.inventory_once()?;

    println!("Allowing field-off effects...");
    reader.write(MemoryBank::User, SENSOR_DATA_MSW, [0x00, 0x00], None)?;
    let stime = std::time::Instant::now();
    let duration = std::time::Duration::from_secs_f32(2.5);
    while stime.elapsed() < duration {}

    println!("Reading measurement...");

    let measurement = reader.read_alt(MemoryBank::User, SENSOR_DATA_MSW, 2, None)?;
    let measurement = u16::from_be_bytes([measurement[0], measurement[1]]);
    let measurement = process_temp(measurement);

    println!("Stopping measurement...");

    reader.write(MemoryBank::User, BAP_MODE_WORD, [0x00, 0x00], None)?;

    let stime = std::time::Instant::now();
    let duration = std::time::Duration::from_secs_f32(0.5);
    while stime.elapsed() < duration {}
    reader.inventory(200, Box::new(|_| ()))?;

    println!("Got temperature: {measurement} °C");

    Ok(())
}
