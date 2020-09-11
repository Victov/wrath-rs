use async_std::net::{TcpStream};
use async_std::prelude::*;
use podio::{ReadPodExt, WritePodExt, LittleEndian};
use std::io::Write;
use num_bigint::{BigUint};
use num_traits::{Zero};
use anyhow::*;
use num_bigint::RandBigInt;
use sha1::Digest;
use super::constants;
use wrath_auth_db::AuthDatabase;

#[derive(Default, Clone)]
pub struct LoginNumbers
{
    pub n : BigUint,
    pub g : BigUint,
    pub b : BigUint,
    pub bb: BigUint,
    pub v : BigUint,
    pub s : BigUint,
    pub username : String,
}

pub async fn handle_logon_challenge(stream : &mut TcpStream, buf : &[u8], auth_database:&std::sync::Arc<AuthDatabase>)
    -> Result<LoginNumbers>
{
    use num_traits::cast::FromPrimitive;

    let mut logindata : LoginNumbers = LoginNumbers::default();

    let mut reader = std::io::Cursor::new(buf);
    let _cmd = reader.read_u8()?;
    let _error = reader.read_u8()?;
    let _size = reader.read_u16::<LittleEndian>()?;
    let _gamename = reader.read_exact(4)?;
    let version1 = reader.read_u8()?;
    let version2 = reader.read_u8()?;
    let version3 = reader.read_u8()?;
    let build = reader.read_u16::<LittleEndian>()?;
    let _platform = reader.read_u32::<LittleEndian>()?;
    let _os = reader.read_u32::<LittleEndian>()?;
    let _country = reader.read_u32::<LittleEndian>()?;
    let _timezone_bias = reader.read_u32::<LittleEndian>()?;
    let ip = reader.read_exact(4)?;
    let username_length = reader.read_u8()? as usize;
    let username_bytes = reader.read_exact(username_length)?;
    let username = std::str::from_utf8(&username_bytes)?;
    logindata.username = username.to_string();

    println!("logon challenge for user {} on ip {}.{}.{}.{}, version {}.{}.{}:{}", username, ip[0], ip[1], ip[2], ip[3], version1, version2, version3, build);

    let account = (*auth_database).get_account_by_username(&username).await;

    if account.is_err()
    {
        reject_login(stream, constants::AuthResult::FailUnknownAccount).await?;
        return Err(anyhow!("Username not found in database"));
    }

    let account = account?;

    if account.banned != 0
    {
        reject_login(stream, constants::AuthResult::FailBanned).await?;
        return Err(anyhow!("Account was banned, refusing login"));
    }
    
    logindata.g = BigUint::from_u64(7u64).unwrap();
    logindata.n = BigUint::parse_bytes(b"894B645E89E1535BBDAD5B8B290650530801B18EBFBF5E8FAB3C82872A3E9BB7", 16)
        .ok_or_else(|| { anyhow!("Failed to parse N") })?;
    let n_bytes = logindata.n.to_bytes_le();
    assert!(n_bytes.len() == 32);

    if account.v.is_empty() || account.s.is_empty()
    {
        let (v, s) = generate_vs(&account.sha_pass_hash, &logindata.g, &logindata.n).await?;
        (*auth_database).set_account_v_s(account.id, &v.to_str_radix(16), &s.to_str_radix(16)).await?;

        logindata.v = v;
        logindata.s = s;

        //println!("v={}", logindata.v.to_str_radix(16));
        //println!("s={}", logindata.s.to_str_radix(16));
    }
    else
    {
        logindata.v = BigUint::parse_bytes(account.v.as_bytes(), 16).ok_or_else(|| { anyhow!("Failed to parse v from database") })?;
        logindata.s = BigUint::parse_bytes(account.s.as_bytes(), 16).ok_or_else(|| { anyhow!("Failed to parse s from database") })?;
    }
    let s_bytes = logindata.s.to_bytes_le();
    assert!(s_bytes.len() == 32);
   
    logindata.b = rand::thread_rng().gen_biguint(19 * 8);
    //logindata.b = BigUint::from_str_radix("6dd94824b210c7b205974f9f53e9539ed459991932b0ca50f6a1bdc839c5c486", 16)?;
    //println!("b={}", logindata.b.to_str_radix(16));

    let gmod = logindata.g.modpow(&logindata.b, &logindata.n); 
    logindata.bb = ((&logindata.v * 3u32) + gmod) % &logindata.n;
    //println!("B={}", logindata.bb.to_str_radix(16));
    let bb_bytes = logindata.bb.to_bytes_le();
    assert!(bb_bytes.len() == 32);
    //bb_bytes.resize(32, 0);

    //let unk : BigUint = rand::thread_rng().gen_biguint(16 * 8);
    //let unk = BigUint::from_str_radix("383855f209cf4e2267e3fc317cf6fb6f", 16)?;
    //println!("unk={}", unk.to_str_radix(16));
    
    //let unk_bytes = unk.to_bytes_le();
    let unk_bytes = [0u8;16];
    assert!(unk_bytes.len() == 16);
     
    let return_buf : Vec<u8> = Vec::new();
    let mut writer = std::io::Cursor::new(return_buf);
    writer.write_u8(0)?; //AUTH_LOGON_CHALLENGE
    writer.write_u8(0)?;
    writer.write_u8(constants::AuthResult::Success as u8)?;
    writer.write(&bb_bytes.as_slice())?;
    writer.write_u8(1)?;
    writer.write_u8(7)?; // value of g in one byte
    writer.write_u8(32)?;
    writer.write(n_bytes.as_slice())?;
    writer.write(s_bytes.as_slice())?;
    writer.write(&unk_bytes)?;
    writer.write_u8(0)?;
    stream.write(&writer.into_inner()).await?;
    stream.flush().await?;
    
    Ok(logindata)
}

pub async fn generate_vs(hashed_password : &String, g : &BigUint, n : &BigUint) -> Result<(BigUint, BigUint)>
{
    let s : BigUint = rand::thread_rng().gen_biguint(256);
    //let s = BigUint::from_str_radix("16db0802870c05966e6ff6f80dd13f689c0067ce77ae233c15f86f316d919885", 16)?;
    
    let pw = BigUint::parse_bytes(hashed_password.as_bytes(), 16).unwrap();
    
    let mut hasher = sha1::Sha1::new();
    hasher.update(s.to_bytes_le());
    hasher.update(pw.to_bytes_be());
    let hashed = hasher.finalize();
    let x : BigUint = BigUint::from_bytes_le(hashed.as_slice());

    let v : BigUint = g.modpow(&x, n);

    Ok((v, s))
}

async fn reject_login(stream : &mut TcpStream, reason: constants::AuthResult) -> Result<()>
{
    let return_buf : Vec<u8> = Vec::new();
    let mut writer = std::io::Cursor::new(return_buf);
    writer.write_u8(0u8)?; //AUTH_LOGON_CHALLENGE
    writer.write_u8(0u8)?;
    writer.write_u8(reason as u8)?; 
    stream.write(&writer.into_inner()).await?;
    stream.flush().await?;
    Ok(())
}

pub async fn handle_logon_proof(stream : &mut TcpStream, buf: &[u8], logindata : &LoginNumbers, auth_database: &std::sync::Arc<AuthDatabase>) -> Result<()>
{
    println!("logon proof");
    if logindata.n.is_zero()
    {
        return Err(anyhow!("Trying to handle login proof but no numbers have been generated from the challenge. Hacking?"));
    }
    
    let mut reader = std::io::Cursor::new(buf);
    let _cmd = reader.read_u8()?;
    let a_bytes = reader.read_exact(32)?;
    let m_one_bytes = reader.read_exact(20)?;
    let _crc_hash_bytes = reader.read_exact(20)?;
    let _number_of_keys = reader.read_u8()?;
    let _security_flags = reader.read_u8()?;

    let a : BigUint = BigUint::from_bytes_le(&a_bytes);
    //let a = BigUint::from_str_radix("3e7738d6735bac8dfa29a261635bc500cb396a1115a7cf8c62ddb75b045e89ec", 16)?;
    //let a_bytes = a.to_bytes_le();
    //println!("A={}", a.to_str_radix(16));
    if (&a % &logindata.n).is_zero()
    {
        return Err(anyhow!("a%N cannot be zero according to SRP protocol"));
    }

    let mut hasher = sha1::Sha1::new();
    hasher.update(&a.to_bytes_le());
    hasher.update(&logindata.bb.to_bytes_le());
    let hashed = hasher.finalize_reset();
    let u = BigUint::from_bytes_le(hashed.as_slice());
    //println!("u={}", u.to_str_radix(16));
    let s = (&a * (logindata.v.modpow(&u, &logindata.n))).modpow(&logindata.b, &logindata.n);
    //println!("S={}", s.to_str_radix(16));
    let s_bytes = s.to_bytes_le();
    assert!(s_bytes.len() == 32);
    
    let t1 = &mut [0; 16];
    for i in 0 .. 16
    {
        t1[i] = s_bytes[i*2];
    }

    hasher.update(&t1);
    let pre_vk = hasher.finalize_reset();
    assert!(pre_vk.len() == 20);

    let vk = &mut [0;40];
    for i in 0 .. 20
    {
        vk[i*2] = pre_vk[i];
    }

    for i in 0 .. 16
    {
        t1[i] = s_bytes[i * 2 + 1];
    }
    assert!(t1.len() == 16);
    
    hasher.update(&t1);
    let pre_vk2 = hasher.finalize_reset();
    
    for i in 0 .. 20
    {
        vk[i * 2 + 1] = pre_vk2[i];
    }

    let k = BigUint::from_bytes_le(vk);
    //println!("k={}", k.to_str_radix(16));
    
    hasher.update(logindata.n.to_bytes_le());
    let mut n_hash = hasher.finalize_reset();
    assert!(n_hash.len() == 20);
    
    hasher.update(logindata.g.to_bytes_le());
    assert!(logindata.g.to_bytes_le().len() == 1);
    let g_hash = hasher.finalize_reset();

    for i in 0 .. 20
    {
        n_hash[i] ^= g_hash[i]; 
    }

    let t3 = BigUint::from_bytes_le(&n_hash);
    hasher.update(&logindata.username);
    let t4 = hasher.finalize_reset();
    assert!(t4.len() == 20);

    hasher.update(&t3.to_bytes_le()); 
    hasher.update(&t4);
    hasher.update(&logindata.s.to_bytes_le());
    hasher.update(&a_bytes);
    hasher.update(&logindata.bb.to_bytes_le());
    hasher.update(&k.to_bytes_le()); 
    let m_bytes = hasher.finalize_reset();
    let m = BigUint::from_bytes_le(&m_bytes);
    let _m1 = BigUint::from_bytes_le(&m_one_bytes);

    hasher.update(&a_bytes);
    hasher.update(&m_bytes);
    hasher.update(&k.to_bytes_le());
    let m2_bytes = hasher.finalize_reset();
    let m2 = BigUint::from_bytes_be(&m2_bytes);

    //println!("m : {}\nm1: {}\nm2: {}", m.to_str_radix(16), m1.to_str_radix(16), m2.to_str_radix(16));
    if m.to_bytes_le() != m_one_bytes
    {
        stream.write(&[1, constants::AuthResult::FailNoAccess as u8, 3, 0]).await?;
        stream.flush().await?;
        return Err(anyhow!("Wrong password"));
    }
    
    (*auth_database).set_account_sessionkey(&logindata.username, &k.to_str_radix(16)).await?;

    let write_buf = Vec::<u8>::new();
    let mut writer = std::io::Cursor::new(write_buf);
    writer.write_u8(1)?; //AUTH_LOGON_PROOF
    writer.write_u8(0)?; //Proof success
    writer.write(&m2.to_bytes_be())?;
    writer.write_u32::<LittleEndian>(0u32)?;
    writer.write_u32::<LittleEndian>(0u32)?;
    writer.write_u16::<LittleEndian>(0u16)?;
    stream.write(&writer.into_inner()).await?;
    stream.flush().await?;

    Ok(())
}
