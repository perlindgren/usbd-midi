use crate::data::usb::constants::*;
use crate::data::usb_midi::usb_midi_event_packet::UsbMidiEventPacket;
use usb_device::class_prelude::*;
use usb_device::Result;

/// Note we are using MidiIn/out here to refer to the fact that
/// the host sees it as a midi in/out respectively
/// This class allows you to send and receive midi event packages
/// (Transfer endpoints not supported)

pub struct MidiClass<'a, B: UsbBus> {
    standard_ac: InterfaceNumber,
    standard_mc: InterfaceNumber,
    standard_bulkin: EndpointIn<'a, B>,   // in, send to host
    standard_bulkout: EndpointOut<'a, B>, // out, receive from host
}

impl<B: UsbBus> MidiClass<'_, B> {
    /// Creates a new MidiClass with the provided UsbBus
    pub fn new(alloc: &UsbBusAllocator<B>) -> MidiClass<'_, B> {
        MidiClass {
            standard_ac: alloc.interface(),
            standard_mc: alloc.interface(),
            standard_bulkin: alloc.bulk(64),
            standard_bulkout: alloc.bulk(64),
        }
    }

    pub fn send_message(&mut self, usb_midi: UsbMidiEventPacket) -> Result<usize> {
        let bytes: [u8; 4] = usb_midi.into();
        self.standard_bulkin.write(&bytes)
    }

    pub fn get_message_raw(&mut self, bytes: &mut [u8]) -> Result<usize> {
        self.standard_bulkout.read(bytes)
    }
}

impl<B: UsbBus> UsbClass<B> for MidiClass<'_, B> {
    fn get_configuration_descriptors(&self, writer: &mut DescriptorWriter) -> Result<()> {
        // AUDIO CONTROL STANDARD

        // A single AudioControl (AC) interface can serve several audio and midi streams
        // which together forms an Audio Interface Collection (AIC)

        // MIDI Data is transferred over the USB in 32-bit USB-MIDI Event Packets,
        // with the first 4 bits used to designate the appropriate Embedded MIDI Jack.

        writer.interface(
            self.standard_ac,
            USB_AUDIO_CLASS,
            USB_AUDIOCONTROL_SUBCLASS,
            0, //no protocol,
        )?;

        // AUDIO CONTROL EXTRA INFO
        // USB Device Class Definition for MIDI Devices, Section B.3.2
        writer.write(
            CS_INTERFACE,
            &[
                HEADER_SUBTYPE,
                // Revision of class specification - 1.0, 0x0100
                0x00, // Lsb
                0x01, // Msb
                // Total Size, 0x0009
                0x09, // Lsb
                0x00, // Msb
                0x01, // Number of streaming interfaces
                0x01, // MIDIStreaming interface 1 belongs to this AC interface
            ],
        )?;

        // Streaming Standard
        // USB Device Class Definition for MIDI Devices, Section B.4.1

        writer.interface(
            self.standard_mc,
            USB_AUDIO_CLASS,
            USB_MIDISTREAMING_SUBCLASS,
            0, //no protocol
        )?;

        // Streaming extra info

        writer.write(
            CS_INTERFACE,
            &[
                MS_HEADER_SUBTYPE,
                // Revision of class specification - 1.0, 0x0100
                0x00, // Lsb
                0x01, // Msb
                // Total Size
                (0x07 + MIDI_OUT_SIZE + MIDI_IN_SIZE + 2 * (EP_SIZE + EP_SIZE)), // Lsb
                0x00,                                                            // Msb
            ],
        )?;

        // JACKS

        // Midi out from the device to Midi in on the host
        const MIDI_OUT_SIZE: u8 = 0x09;
        writer.write(
            CS_INTERFACE,
            &[
                MIDI_OUT_JACK_SUBTYPE, // bDescriptorSubtype
                EMBEDDED,              // bJackType
                0x01,                  // bJackID, 1
                0x01,                  // bNrInputPins, 1
                0x01,                  // BaSourceID, 1
                0x01,                  // BaSourcePin, 1
                0x00,
            ],
        )?;

        // Midi in to the device from Midi out on the host
        const MIDI_IN_SIZE: u8 = 0x06;
        writer.write(
            CS_INTERFACE,
            &[
                MIDI_IN_JACK_SUBTYPE, // bDescriptorSubtype
                EMBEDDED,             // bJackType
                0x02,                 // bJackID, 2
                0x00,                 // unused
            ],
        )?;
        const EP_SIZE: u8 = 0x09;
        writer.endpoint(&self.standard_bulkin)?;
        const EP_CLASS_SIZE: u8 = 0x05;

        writer.write(CS_ENDPOINT, &[MS_GENERAL, 0x01, 0x01])?;
        writer.endpoint(&self.standard_bulkout)?;
        writer.write(CS_ENDPOINT, &[MS_GENERAL, 0x01, 0x01])?;

        Ok(())
    }
}
