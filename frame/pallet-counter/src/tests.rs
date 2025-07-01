//! Tests for the security fixes in pallet-counter

use super::*;
use crate::mock::*;
use frame_support::{assert_err, assert_ok, traits::Get};

#[test]
fn test_enhanced_suspicious_content_detection() {
    // Test basic blacklisted words
    assert!(Pallet::<Test>::contains_suspicious_content("this is a scam"));
    assert!(Pallet::<Test>::contains_suspicious_content("fraud alert"));
    assert!(Pallet::<Test>::contains_suspicious_content("illegal activity"));
    
    // Test leetspeak bypasses
    assert!(Pallet::<Test>::contains_suspicious_content("sc4m alert"));
    assert!(Pallet::<Test>::contains_suspicious_content("fr4ud here"));
    assert!(Pallet::<Test>::contains_suspicious_content("h4ck attempt"));
    assert!(Pallet::<Test>::contains_suspicious_content("ph1shing link"));
    
    // Test separator bypasses
    assert!(Pallet::<Test>::contains_suspicious_content("s_c_a_m"));
    assert!(Pallet::<Test>::contains_suspicious_content("s-c-a-m"));
    assert!(Pallet::<Test>::contains_suspicious_content("s.c.a.m"));
    assert!(Pallet::<Test>::contains_suspicious_content("s c a m"));
    
    // Test crypto-specific scams
    assert!(Pallet::<Test>::contains_suspicious_content("free_crypto giveaway"));
    assert!(Pallet::<Test>::contains_suspicious_content("airdrop scam"));
    assert!(Pallet::<Test>::contains_suspicious_content("rugpull warning"));
    assert!(Pallet::<Test>::contains_suspicious_content("seed_phrase request"));
    
    // Test excessive symbols (spam)
    assert!(Pallet::<Test>::contains_suspicious_content("!!!@@@###$$$%%%"));
    
    // Test repeated characters (spam)
    assert!(Pallet::<Test>::contains_suspicious_content("heyyyyyyy"));
    assert!(Pallet::<Test>::contains_suspicious_content("woooooooow"));
    
    // Test legitimate messages that should pass
    assert!(!Pallet::<Test>::contains_suspicious_content("hello world"));
    assert!(!Pallet::<Test>::contains_suspicious_content("payment for services"));
    assert!(!Pallet::<Test>::contains_suspicious_content("thank you for help"));
    assert!(!Pallet::<Test>::contains_suspicious_content("monthly subscription"));
}

#[test]
fn test_enhanced_ipv4_validation() {
    // Valid IPv4 addresses
    assert!(Pallet::<Test>::contains_ipv4_address("192.168.1.1"));
    assert!(Pallet::<Test>::contains_ipv4_address("10.0.0.1"));
    assert!(Pallet::<Test>::contains_ipv4_address("172.16.0.1"));
    assert!(Pallet::<Test>::contains_ipv4_address("8.8.8.8"));
    assert!(Pallet::<Test>::contains_ipv4_address("255.255.255.255"));
    assert!(Pallet::<Test>::contains_ipv4_address("0.0.0.0"));
    
    // Invalid IPv4 addresses that should be rejected
    assert!(!Pallet::<Test>::contains_ipv4_address("256.1.1.1")); // Out of range
    assert!(!Pallet::<Test>::contains_ipv4_address("192.168.1")); // Too few octets
    assert!(!Pallet::<Test>::contains_ipv4_address("192.168.1.1.1")); // Too many octets
    assert!(!Pallet::<Test>::contains_ipv4_address("192.168.01.1")); // Leading zeros
    assert!(!Pallet::<Test>::contains_ipv4_address("192.168..1")); // Empty octet
    assert!(!Pallet::<Test>::contains_ipv4_address("192.168.a.1")); // Non-numeric
    assert!(!Pallet::<Test>::contains_ipv4_address("")); // Empty string
}

#[test]
fn test_enhanced_ipv6_validation() {
    // Valid IPv6 addresses
    assert!(Pallet::<Test>::contains_ipv6_address("2001:0db8:85a3:0000:0000:8a2e:0370:7334"));
    assert!(Pallet::<Test>::contains_ipv6_address("2001:db8:85a3::8a2e:370:7334")); // Compressed
    assert!(Pallet::<Test>::contains_ipv6_address("::1")); // Loopback
    assert!(Pallet::<Test>::contains_ipv6_address("::")); // All zeros
    assert!(Pallet::<Test>::contains_ipv6_address("fe80::1")); // Link-local
    assert!(Pallet::<Test>::contains_ipv6_address("2001:db8::1"));
    
    // Invalid IPv6 addresses that should be rejected
    assert!(!Pallet::<Test>::contains_ipv6_address("2001:0db8:85a3::8a2e::7334")); // Multiple ::
    assert!(!Pallet::<Test>::contains_ipv6_address("2001:0db8:85a3:0000:0000:8a2e:0370:7334:extra")); // Too many groups
    assert!(!Pallet::<Test>::contains_ipv6_address("2001:0db8:85a3g:0000:0000:8a2e:0370:7334")); // Invalid hex
    assert!(!Pallet::<Test>::contains_ipv6_address("2001:0db8:85a33:0000:0000:8a2e:0370:7334")); // Group too long
    assert!(!Pallet::<Test>::contains_ipv6_address(":::")); // Triple colon
    assert!(!Pallet::<Test>::contains_ipv6_address(":2001:db8::1")); // Leading single colon
    assert!(!Pallet::<Test>::contains_ipv6_address("2001:db8::1:")); // Trailing single colon
    assert!(!Pallet::<Test>::contains_ipv6_address("2001::db8::1")); // Multiple compression
    
    // Edge cases that should be rejected
    assert!(!Pallet::<Test>::contains_ipv6_address("a:")); // Too short but was previously accepted
    assert!(!Pallet::<Test>::contains_ipv6_address("a:b")); // Too few groups
    assert!(!Pallet::<Test>::contains_ipv6_address("12345::")); // Group too long
}

#[test]
fn test_ip_address_detection_in_messages() {
    // Messages containing IPv4 should be rejected
    assert!(Pallet::<Test>::contains_ip_address("Visit 192.168.1.1 for more"));
    assert!(Pallet::<Test>::contains_ip_address("Connect to 8.8.8.8"));
    
    // Messages containing IPv6 should be rejected
    assert!(Pallet::<Test>::contains_ip_address("Server at 2001:db8::1"));
    assert!(Pallet::<Test>::contains_ip_address("Use ::1 for localhost"));
    
    // Messages without IP addresses should pass
    assert!(!Pallet::<Test>::contains_ip_address("Hello world"));
    assert!(!Pallet::<Test>::contains_ip_address("Version 1.2.3 is available")); // Not IP
    assert!(!Pallet::<Test>::contains_ip_address("Price: $1.99")); // Not IP
}

#[test]
fn test_text_normalization() {
    // Test that normalization handles various bypass attempts
    assert_eq!(
        Pallet::<Test>::normalize_text("sc@m"),
        "scam"
    );
    assert_eq!(
        Pallet::<Test>::normalize_text("fr4ud"),
        "fraud"
    );
    assert_eq!(
        Pallet::<Test>::normalize_text("h4ck3r"),
        "hacker"
    );
    assert_eq!(
        Pallet::<Test>::normalize_text("s_c_a_m"),
        "scam"
    );
    assert_eq!(
        Pallet::<Test>::normalize_text("S-C-A-M"),
        "scam"
    );
    assert_eq!(
        Pallet::<Test>::normalize_text("ph1sh1ng"),
        "phishing"
    );
}

#[cfg(test)]
mod mock {
    use super::*;
    use frame_support::{
        derive_impl, parameter_types,
        traits::{ConstU16, ConstU32, ConstU64},
        weights::Weight,
    };
    use sp_runtime::{
        traits::{BlakeTwo256, IdentityLookup},
        BuildStorage,
    };

    type Block = frame_system::mocking::MockBlock<Test>;

    frame_support::construct_runtime!(
        pub enum Test {
            System: frame_system,
            Balances: pallet_balances,
            Counter: crate,
        }
    );

    #[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
    impl frame_system::Config for Test {
        type Block = Block;
        type AccountData = pallet_balances::AccountData<u64>;
    }

    #[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
    impl pallet_balances::Config for Test {
        type AccountStore = System;
    }

    impl crate::Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type SubstrateCurrency = Balances;
        type EvmCurrency = Balances;
        type BackendOrigin = frame_system::EnsureRoot<Self::AccountId>;
    }

    pub fn new_test_ext() -> sp_io::TestExternalities {
        system::GenesisConfig::<Test>::default().build_storage().unwrap().into()
    }
}