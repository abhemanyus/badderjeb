use krpc_mars::client::CallHandle;
use krpc_mars::codec::RPCEncodable;
use krpc_mars::codec::RPCExtractable;
use krpc_mars::krpc;
use krpc_mars::protobuf;

use std::fmt;

#[derive(Copy, Clone)]
pub struct Camera {
    id: u32,
}

impl fmt::Debug for Camera {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Camera({})", self.id)
    }
}

impl RPCEncodable for Camera {
    fn encode(
        &self,
        output: &mut protobuf::CodedOutputStream,
    ) -> Result<(), protobuf::ProtobufError> {
        self.id.encode(output)
    }
}

impl RPCExtractable for Camera {
    fn extract_value(
        input: &mut protobuf::CodedInputStream,
    ) -> Result<Self, protobuf::ProtobufError> {
        let id = RPCExtractable::extract_value(input)?;
        Ok(Camera { id })
    }
}

#[allow(dead_code)]
impl Camera {
    /// <doc> <summary> Get an image. Returns an empty byte array on failure. </summary> </doc>
    pub fn get_image(&self) -> CallHandle<Vec<u8>> {
        let mut proc_call = krpc::ProcedureCall::new();
        proc_call.set_service(String::from("DockingCamera"));
        proc_call.set_procedure(String::from("Camera_get_Image"));

        let mut arguments = protobuf::RepeatedField::<krpc::Argument>::new();

        let mut arg0 = krpc::Argument::new();
        arg0.set_position(0);
        arg0.set_value(self.id.encode_to_bytes().unwrap());
        arguments.push(arg0);
        proc_call.set_arguments(arguments);

        CallHandle::new(proc_call)
    }
    /// <doc> <summary> Get the part containing this camera. </summary> </doc>
    pub fn get_part(&self) -> CallHandle<super::space_center::Part> {
        let mut proc_call = krpc::ProcedureCall::new();
        proc_call.set_service(String::from("DockingCamera"));
        proc_call.set_procedure(String::from("Camera_get_Part"));

        let mut arguments = protobuf::RepeatedField::<krpc::Argument>::new();

        let mut arg0 = krpc::Argument::new();
        arg0.set_position(0);
        arg0.set_value(self.id.encode_to_bytes().unwrap());
        arguments.push(arg0);
        proc_call.set_arguments(arguments);

        CallHandle::new(proc_call)
    }
}

#[allow(dead_code)]
/// <doc> <summary> Get a Camera part. </summary> </doc>
pub fn camera(p_part: super::space_center::Part) -> CallHandle<super::docking_camera::Camera> {
    let mut proc_call = krpc::ProcedureCall::new();
    proc_call.set_service(String::from("DockingCamera"));
    proc_call.set_procedure(String::from("Camera"));

    let mut arguments = protobuf::RepeatedField::<krpc::Argument>::new();

    let mut arg0 = krpc::Argument::new();
    arg0.set_position(0);
    arg0.set_value(p_part.encode_to_bytes().unwrap());
    arguments.push(arg0);
    proc_call.set_arguments(arguments);

    CallHandle::new(proc_call)
}

#[allow(dead_code)]
/// <doc> <summary> Check if the Camera API is available. </summary> </doc>
pub fn get_available() -> CallHandle<bool> {
    let mut proc_call = krpc::ProcedureCall::new();
    proc_call.set_service(String::from("DockingCamera"));
    proc_call.set_procedure(String::from("get_Available"));

    CallHandle::new(proc_call)
}
