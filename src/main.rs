//! ```text
//!  __          __                   _                       _                            _       _
//!  \ \        / /                  (_)                     | |                          (_)     | |
//!   \ \  /\  / /____      __  _ __  _  ___ ___    ___ _ __ | |_ _ __ _   _   _ __   ___  _ _ __ | |_
//!    \ \/  \/ / _ \ \ /\ / / | '_ \| |/ __/ _ \  / _ \ '_ \| __| '__| | | | | '_ \ / _ \| | '_ \| __|
//!     \  /\  / (_) \ V  V /  | | | | | (_|  __/ |  __/ | | | |_| |  | |_| | | |_) | (_) | | | | | |_
//!      \/  \/ \___/ \_/\_/   |_| |_|_|\___\___|  \___|_| |_|\__|_|   \__, | | .__/ \___/|_|_| |_|\__|
//!                                                                     __/ | | |
//!                                                                    |___/  |_|
//! ```

#![no_std]
#![no_main]
#![warn(warnings, clippy::cargo)]
#![feature(abi_efiapi)]
#![feature(allocator_api)]
#![feature(core_intrinsics)]

use amd64::registers::msr::Msr;
use log::{error, info};
use raw_cpuid::CpuId;
use uefi::{
    prelude::{entry, Boot, Handle, Status, SystemTable},
    ResultExt,
};

#[entry]
fn efi_main(_image: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).expect_success("Failed to initialize utilities");

    let cpuid = CpuId::new();
    let has_svm = cpuid
        .get_extended_processor_and_feature_identifiers()
        .expect("Failed to get CPUID EXT Processor and Feature IDs")
        .has_svm();

    if has_svm {
        let mut vm_cr = unsafe { amd64::registers::msr::VmCr::read() };
        let svm_features = cpuid.get_svm_info().expect("Failed to get SVM features");

        if vm_cr.disabled() {
            let has_svm_lock = svm_features.has_svm_lock();

            if has_svm_lock && vm_cr.locked() {
                error!("AMD-V is disabled and locked.");
                return Status::UNSUPPORTED;
            }

            info!("AMD-V is disabled but unlocked, enabling it.");
            vm_cr.set_disabled(false);
            unsafe {
                vm_cr.write();
            }
            info!("Done.");
        }

        info!(
            "AMD-V is enabled. NRIP save support: {}, Nested paging support: {}",
            svm_features.has_nrip(),
            svm_features.has_nested_paging()
        );

        Status::SUCCESS
    } else {
        error!("AMD-V unsupported.");
        Status::UNSUPPORTED
    }
}
