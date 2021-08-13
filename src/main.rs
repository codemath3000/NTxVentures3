#[macro_use]
extern crate redis;

use redis::{Value, Client, Cmd, cmd, ConnectionLike, RedisResult, Connection, FromRedisValue, PubSub, ToRedisArgs, RedisWrite};
use std::collections::{HashSet, HashMap, BTreeMap};
use bytes::Bytes;
use std::hash::{Hash, Hasher};
use std::cmp::Ordering;
use std::collections::hash_map::{RandomState, DefaultHasher};

struct RedisValue {
    int_value: i64,
    string_value: String,
    vec_value: Vec<RedisValue>,
    hash_set_value: HashSet<RedisValue>,
    hash_map_value: HashMap<RedisValue, RedisValue>,
    value_index: i32,
}

impl RedisValue {
    fn test_redis_value_conversion<T: FromRedisValue>(input_value: &Value) -> bool {
        let test_result: RedisResult<T> = FromRedisValue::from_redis_value(input_value);
        return test_result.is_ok();
    }

    fn convert_redis_value<T: FromRedisValue>(input_value: &Value) -> T {
        return FromRedisValue::from_redis_value(input_value).unwrap();
    }

    fn write_variable_to_args<T: ToRedisArgs, W: ?Sized + RedisWrite>(input: &T, output: &mut W) {
        let redis_args: Vec<Vec<u8>> = input.to_redis_args();
        for redis_arg in redis_args {
            output.write_arg(redis_arg.as_slice());
        }
    }

    fn hash_to_tree<T: Ord + Clone, W: Ord + Clone>(input: HashMap<T, W>) -> BTreeMap<T, W> {
        let mut output: BTreeMap<T, W> = BTreeMap::new();
        for (input_key, input_value) in input.iter() {
            output.insert(input_key.clone(), input_value.clone());
        }
        return output;
    }
}

impl Hash for RedisValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self.value_index {
            0 => {
                self.int_value.hash(state);
            },
            1 => {
                self.string_value.hash(state);
            },
            2 => {
                self.vec_value.hash(state);
            },
            3 => {
                self.hash_set_value.to_redis_args().hash(state);
            },
            4 => {
                Self::hash_to_tree(self.hash_map_value.clone()).to_redis_args().hash(state);
            },
            _ => {
                panic!("Invalid type for RedisValue");
            }
        }
    }
}

impl PartialOrd for RedisValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let mut hasher: DefaultHasher = DefaultHasher::new();
        self.hash(&mut hasher);
        let self_hash: u64 = hasher.finish();
        other.hash(&mut hasher);
        let other_hash: u64 = hasher.finish();
        if self_hash < other_hash { return Option::Some(Ordering::Less); }
        else if self_hash > other_hash { return Option::Some(Ordering::Greater); }
        else { return Option::Some(Ordering::Equal); }
    }
}

impl Ord for RedisValue {
    fn cmp(&self, other: &Self) -> Ordering {
        return self.partial_cmp(other).unwrap();
    }
}

impl Clone for RedisValue {
    fn clone(&self) -> Self {
        Self {
            int_value: i64::from(self.int_value),
            string_value: self.string_value.clone(),
            vec_value: self.vec_value.clone(),
            hash_set_value: self.hash_set_value.clone(),
            hash_map_value: self.hash_map_value.clone(),
            value_index: self.value_index,
        }
    }
}

impl PartialEq for RedisValue {
    fn eq(&self, other: &Self) -> bool {
        match self.value_index {
            0 => {
                self.int_value == other.int_value
            },
            1 => {
                self.string_value.eq(&other.string_value)
            },
            2 => {
                self.vec_value.eq(&other.vec_value)
            },
            3 => {
                self.hash_set_value.to_redis_args().eq(&other.hash_set_value.to_redis_args())
            },
            4 => {
                Self::hash_to_tree(self.hash_map_value.clone()).to_redis_args().eq(&Self::hash_to_tree(other.hash_map_value.clone()).to_redis_args())
            },
            _ => {
                panic!("Invalid type for RedisValue");
            }
        }
    }
}

impl Eq for RedisValue {

}

impl ToRedisArgs for RedisValue {
    fn write_redis_args<T>(&self, out: &mut T) where T: ?Sized + RedisWrite {
        match self.value_index {
            0 => {
                Self::write_variable_to_args(&self.int_value, out);
            },
            1 => {
                Self::write_variable_to_args(&self.string_value, out);
            },
            2 => {
                Self::write_variable_to_args(&self.vec_value, out);
            },
            3 => {
                Self::write_variable_to_args(&self.hash_set_value, out);
            },
            4 => {
                Self::write_variable_to_args(&Self::hash_to_tree(self.hash_map_value.clone()), out);
            },
            _ => {
                panic!("Invalid type for RedisValue");
            }
        }
    }
}

impl FromRedisValue for RedisValue {
    fn from_redis_value(input_value: &Value) -> RedisResult<RedisValue> {
        let mut int_value: i64 = 0;
        let mut string_value: String = "".to_owned();
        let mut vec_value: Vec<RedisValue> = Vec::new();
        let mut hash_set_value: HashSet<RedisValue> = HashSet::new();
        let mut hash_map_value: HashMap<RedisValue, RedisValue> = HashMap::new();
        let mut value_index: i32 = -1;
        if Self::test_redis_value_conversion::<u8>(input_value) {
            int_value = Self::convert_redis_value::<u8>(input_value) as i64;
            value_index = 0;
        }
        else if Self::test_redis_value_conversion::<u8>(input_value) {
            int_value = Self::convert_redis_value::<u8>(input_value) as i64;
            value_index = 0;
        }
        else if Self::test_redis_value_conversion::<i8>(input_value) {
            int_value = Self::convert_redis_value::<i8>(input_value) as i64;
            value_index = 0;
        }
        else if Self::test_redis_value_conversion::<u16>(input_value) {
            int_value = Self::convert_redis_value::<u16>(input_value) as i64;
            value_index = 0;
        }
        else if Self::test_redis_value_conversion::<i16>(input_value) {
            int_value = Self::convert_redis_value::<i16>(input_value) as i64;
            value_index = 0;
        }
        else if Self::test_redis_value_conversion::<u32>(input_value) {
            int_value = Self::convert_redis_value::<u32>(input_value) as i64;
            value_index = 0;
        }
        else if Self::test_redis_value_conversion::<i32>(input_value) {
            int_value = Self::convert_redis_value::<i32>(input_value) as i64;
            value_index = 0;
        }
        else if Self::test_redis_value_conversion::<u64>(input_value) {
            int_value = Self::convert_redis_value::<u64>(input_value) as i64;
            value_index = 0;
        }
        else if Self::test_redis_value_conversion::<i64>(input_value) {
            int_value = Self::convert_redis_value::<i64>(input_value) as i64;
            value_index = 0;
        }
        else if Self::test_redis_value_conversion::<u128>(input_value) {
            int_value = Self::convert_redis_value::<u128>(input_value) as i64;
            value_index = 0;
        }
        else if Self::test_redis_value_conversion::<i128>(input_value) {
            int_value = Self::convert_redis_value::<i128>(input_value) as i64;
            value_index = 0;
        }
        else if Self::test_redis_value_conversion::<usize>(input_value) {
            int_value = Self::convert_redis_value::<usize>(input_value) as i64;
            value_index = 0;
        }
        else if Self::test_redis_value_conversion::<isize>(input_value) {
            int_value = Self::convert_redis_value::<isize>(input_value) as i64;
            value_index = 0;
        }
        else if Self::test_redis_value_conversion::<bool>(input_value) {
            int_value = if Self::convert_redis_value::<bool>(input_value) { 1 } else { 0 };
            value_index = 0;
        }
        else if Self::test_redis_value_conversion::<String>(input_value) {
            string_value = Self::convert_redis_value::<String>(input_value);
            value_index = 1;
        }
        else if Self::test_redis_value_conversion::<Vec<RedisValue>>(input_value) {
            vec_value = Self::convert_redis_value::<Vec<RedisValue>>(input_value);
            value_index = 2;
        }
        else if Self::test_redis_value_conversion::<HashSet<RedisValue>>(input_value) {
            hash_set_value = Self::convert_redis_value::<HashSet<RedisValue>>(input_value);
            value_index = 3;
        }
        else if Self::test_redis_value_conversion::<HashMap<RedisValue, RedisValue>>(input_value) {
            hash_map_value = Self::convert_redis_value::<HashMap<RedisValue, RedisValue>>(input_value);
            value_index = 4;
        }
        else {
            panic!("Unsupported input type")
        }
        return RedisResult::Ok(Self {
            int_value,
            string_value,
            vec_value,
            hash_set_value,
            hash_map_value,
            value_index,
        });
    }
}



struct RedisService {
    redis_client: Client,
}

impl RedisService {
    pub fn setup(cache_url: String) -> Self {
        Self { redis_client: Client::open(cache_url).unwrap() }
    }
    fn get_connection(&self) -> Connection {
        return self.redis_client.get_connection().unwrap();
    }
    pub fn run_command<T: FromRedisValue>(&self, command: String, arguments: &[RedisValue]) -> RedisResult<T> {
        return cmd(command.as_str()).arg(arguments).query::<T>(&mut self.get_connection());
    }
    pub fn listen(&self, channels: &[String]) {
        let mut connection: Connection = self.get_connection();
        let mut connection_pubsub = connection.as_pubsub();
        for channel in channels {
            connection_pubsub.subscribe(channel.as_str());
        }
        loop {
            let pubsub_message = connection_pubsub.get_message().unwrap();
            let message_payload: RedisValue = pubsub_message.get_payload().unwrap();
            // Insert code to do something with the message, then remove the code below.
            println!("Begin Message");
            for redis_arg in message_payload.to_redis_args() {
                for redis_byte in redis_arg {
                    print!("{} ", (redis_byte as u32).to_string());
                }
                println!();
            }
            println!("Message Complete");
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn perform_tests() {
        let service: RedisService = RedisService::setup("redis://138.91.147.234:6379/".to_owned());
        service.run_command::<String>("SET".to_owned(), &[RedisValue::from_redis_value(&Value::Status("keytest".to_owned().into())).unwrap(), RedisValue::from_redis_value(&Value::Status("valuetest".to_owned())).unwrap()]);
        assert_eq!("valuetest".to_owned(), service.run_command::<String>("GET".to_owned(), &[RedisValue::from_redis_value(&Value::Status("keytest".to_owned().into())).unwrap()]).unwrap());
    }
}

fn main() {

}
