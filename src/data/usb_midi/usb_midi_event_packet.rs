use crate::data::usb_midi::cable_number::CableNumber;
use crate::data::usb_midi::code_index_number::CodeIndexNumber;
use crate::data::midi::message::Message;
use crate::data::byte::u4::U4;
use crate::data::midi::message::raw::{Payload,Raw};


/// A packet that communicates with the host
/// Currently supported is sending the specified normal midi
/// message over the supplied cable number
///
/// Each Midi event is 32 bits
/// Byte 0         | Byte 1 | Byte 2 | Byte 3
/// -----------------------------------------
/// Cable  | Code  | MIDI_0 | MIDI_1 | MIDI_2
/// Number | Index |        |        |
///
/// See USB Device Class Definition for MIDI Devices, Section 4
///
pub struct UsbMidiEventPacket {
    pub cable_number : CableNumber,
    pub message: Message
}

impl From<UsbMidiEventPacket> for [u8;4] {
    fn from(value:UsbMidiEventPacket) -> [u8;4] {
        let message= value.message;
        let cable_number = U4::from(value.cable_number);
        let index_number = {
                let code_index = 
                        CodeIndexNumber::find_from_message(&message);
                U4::from(code_index)
        };
        // Byte 0
        let header = U4::combine(cable_number,index_number);

        let raw_midi = Raw::from(message);
        let status = raw_midi.status;

        match raw_midi.payload {
            Payload::Empty => [header,status,0,0],
            Payload::SingleByte(byte) => 
                                [header,status,byte.into(),0],
            Payload::DoubleByte(byte1,byte2) => 
                                    [header,status,byte1.into(),byte2.into()]           
        }
    }
}

impl UsbMidiEventPacket{

    pub fn from_midi(cable:CableNumber, midi:Message)
        -> UsbMidiEventPacket{
        UsbMidiEventPacket{
            cable_number : cable,
            message : midi
        }
    }
}