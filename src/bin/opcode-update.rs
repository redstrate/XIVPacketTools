use XIVPacketTools::{KnownOpCode, KnownOpCodes};
use std::fs;

fn update_opcode(opcode_list: &mut [KnownOpCode], old_opcode: i32, new_opcode: i32) {
    for opcode in opcode_list {
        if opcode.opcode == old_opcode && !opcode.dirty {
            opcode.opcode = new_opcode;
            opcode.dirty = true; // don't allow updating an opcode twice'
            break;
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let opcode_diff: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&args[1]).unwrap()).unwrap();
    let mut old_opcodes: KnownOpCodes =
        serde_json::from_str(&fs::read_to_string(&args[2]).unwrap()).unwrap();

    for opcode_pair in opcode_diff.as_array().unwrap() {
        let old_opcode = i32::from_str_radix(
            opcode_pair["old"]
                .as_array()
                .unwrap()
                .first()
                .unwrap()
                .as_str()
                .unwrap()
                .strip_prefix("0x")
                .unwrap(),
            16,
        )
        .unwrap();
        let new_opcode = i32::from_str_radix(
            opcode_pair["new"]
                .as_array()
                .unwrap()
                .first()
                .unwrap()
                .as_str()
                .unwrap()
                .strip_prefix("0x")
                .unwrap(),
            16,
        )
        .unwrap();

        // currently only checks zone for now:
        update_opcode(
            &mut old_opcodes.server_zone_ipc_type,
            old_opcode,
            new_opcode,
        );
        update_opcode(
            &mut old_opcodes.client_zone_ipc_type,
            old_opcode,
            new_opcode,
        );
    }

    std::fs::write(
        &args[2],
        serde_json::to_string_pretty(&old_opcodes).unwrap(),
    )
    .unwrap();

    println!("Successfully wrote new opcodes!")
}
