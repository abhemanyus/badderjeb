use krpc_mars::krpc as krpc;
use krpc_mars::protobuf as protobuf;
use krpc_mars::client::CallHandle;
use krpc_mars::codec::RPCEncodable;
use krpc_mars::codec::RPCExtractable;

use std::fmt;


#[derive(Copy, Clone)]
pub struct Laser {
    id: u32,
}

impl fmt::Debug for Laser {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Laser({})", self.id)
    }
}

impl RPCEncodable for Laser {
    fn encode(&self, output: &mut protobuf::CodedOutputStream) -> Result<(), protobuf::ProtobufError> {
        self.id.encode(output)
    }
}

impl RPCExtractable for Laser {
    fn extract_value(input: &mut protobuf::CodedInputStream) -> Result<Self, protobuf::ProtobufError> {
        let id = RPCExtractable::extract_value(input)?;
        Ok(Laser { id })
    }
}

#[allow(dead_code)]
impl Laser {
    /// <doc> <summary> Get the point cloud from the LiDAR. Returns an empty list on failure. </summary> </doc>
    pub fn get_cloud(&self, ) -> CallHandle<Vec<f64>> {
        let mut proc_call = krpc::ProcedureCall::new();
        proc_call.set_service(String::from("LiDAR"));
        proc_call.set_procedure(String::from("Laser_get_Cloud"));

        let mut arguments = protobuf::RepeatedField::<krpc::Argument>::new();

        let mut arg0 = krpc::Argument::new();
        arg0.set_position(0);
        arg0.set_value(self.id.encode_to_bytes().unwrap());
        arguments.push(arg0);
        proc_call.set_arguments(arguments);

        CallHandle::new(proc_call)
    }
    /// <doc> <summary> Get the part containing this LiDAR. </summary> </doc>
    pub fn get_part(&self, ) -> CallHandle<super::space_center::Part> {
        let mut proc_call = krpc::ProcedureCall::new();
        proc_call.set_service(String::from("LiDAR"));
        proc_call.set_procedure(String::from("Laser_get_Part"));

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
/// <doc> <summary> Get a LaserDist part. </summary> </doc>
pub fn laser(p_part: super::space_center::Part) -> CallHandle<super::li_dar::Laser> {
    let mut proc_call = krpc::ProcedureCall::new();
    proc_call.set_service(String::from("LiDAR"));
    proc_call.set_procedure(String::from("Laser"));

    let mut arguments = protobuf::RepeatedField::<krpc::Argument>::new();

    let mut arg0 = krpc::Argument::new();
    arg0.set_position(0);
    arg0.set_value(p_part.encode_to_bytes().unwrap());
    arguments.push(arg0);
    proc_call.set_arguments(arguments);

    CallHandle::new(proc_call)
}

#[allow(dead_code)]
/// <doc> <summary> Check if the LaserDist API is available. </summary> </doc>
pub fn get_available() -> CallHandle<bool> {
    let mut proc_call = krpc::ProcedureCall::new();
    proc_call.set_service(String::from("LiDAR"));
    proc_call.set_procedure(String::from("get_Available"));

    CallHandle::new(proc_call)
}

