extern crate erupt;
extern crate vk_mem_erupt;

use erupt::extensions::ext_debug_report::*;
//use std::os::raw::{c_char, c_void};
use erupt::DeviceLoader;
use std::sync::Arc;

fn extension_names() -> Vec<*const i8> {
    vec![EXT_DEBUG_REPORT_EXTENSION_NAME, erupt::extensions::khr_get_physical_device_properties2::KHR_GET_PHYSICAL_DEVICE_PROPERTIES_2_EXTENSION_NAME]
}

// unsafe extern "system" fn vulkan_debug_callback(
//     _: erupt::vk::DebugReportFlagsEXT,
//     _: erupt::vk::DebugReportObjectTypeEXT,
//     _: u64,
//     _: usize,
//     _: i32,
//     _: *const c_char,
//     p_message: *const c_char,
//     _: *mut c_void,
// ) -> u32 {
//     println!("{:?}", ::std::ffi::CStr::from_ptr(p_message));
//     erupt::vk::FALSE
// }

pub struct TestHarness {
    pub device: Arc<erupt::DeviceLoader>,
    pub instance: Arc<erupt::InstanceLoader>,
    pub entry: erupt::EntryLoader,
    pub physical_device: erupt::vk::PhysicalDevice,
    //pub debug_callback: erupt::vk::DebugReportCallbackEXT,
    //pub debug_report_loader: erupt::extensions::ext_debug_report::DebugReport,
}

impl Drop for TestHarness {
    fn drop(&mut self) {
        unsafe {
            self.device.device_wait_idle().unwrap();
            self.device.destroy_device(None);
            //self.debug_report_loader.destroy_debug_report_callback(self.debug_callback, None);
            self.instance.destroy_instance(None);
        }
    }
}
impl TestHarness {
    pub fn new() -> Self {
        let app_name = ::std::ffi::CString::new("vk-mem testing").unwrap();
        let app_info = erupt::vk::ApplicationInfoBuilder::new()
            .application_name(&app_name)
            .application_version(0)
            .engine_name(&app_name)
            .engine_version(0)
            .api_version(erupt::vk::make_api_version(0, 1, 0, 0));

        let layer_names = [::std::ffi::CString::new("VK_LAYER_KHRONOS_validation").unwrap()];
        let layers_names_raw: Vec<*const i8> = layer_names
            .iter()
            .map(|raw_name| raw_name.as_ptr())
            .collect();

        let extension_names_raw = extension_names();
        let create_info = erupt::vk::InstanceCreateInfoBuilder::new()
            .application_info(&app_info)
            .enabled_layer_names(&layers_names_raw)
            .enabled_extension_names(&extension_names_raw);

        let entry = erupt::EntryLoader::new().unwrap();
        let instance: erupt::InstanceLoader = unsafe {
            erupt::InstanceLoader::new(&entry, &create_info, None).expect("Instance creation error")
        };

        // let debug_info = erupt::vk::DebugReportCallbackCreateInfoEXT::builder()
        //     .flags(
        //         erupt::vk::DebugReportFlagsEXT::ERROR
        //             | erupt::vk::DebugReportFlagsEXT::WARNING
        //             | erupt::vk::DebugReportFlagsEXT::PERFORMANCE_WARNING,
        //     )
        //     .pfn_callback(Some(vulkan_debug_callback));

        // let debug_report_loader = DebugReport::new(&entry, &instance);
        // let debug_callback = unsafe {
        //     debug_report_loader
        //         .create_debug_report_callback(&debug_info, None)
        //         .unwrap()
        // };

        let physical_devices = unsafe {
            instance
                .enumerate_physical_devices(None)
                .expect("Physical device error")
        };

        let (physical_device, queue_family_index) = unsafe {
            physical_devices
                .iter()
                .map(|physical_device| {
                    instance
                        .get_physical_device_queue_family_properties(*physical_device, None)
                        .iter()
                        .enumerate()
                        .filter_map(|(index, _)| Some((*physical_device, index)))
                        .nth(0)
                })
                .filter_map(|v| v)
                .nth(0)
                .expect("Couldn't find suitable device.")
        };

        let priorities = [1.0];

        let queue_info = [erupt::vk::DeviceQueueCreateInfoBuilder::new()
            .queue_family_index(queue_family_index as u32)
            .queue_priorities(&priorities)];

        let layer_names = [::std::ffi::CString::new("VK_KHR_portability_subset").unwrap()];
        let layers_names_raw: Vec<*const i8> = layer_names
            .iter()
            .map(|raw_name| raw_name.as_ptr())
            .collect();

        let device_create_info = erupt::vk::DeviceCreateInfoBuilder::new()
            .queue_create_infos(&queue_info)
            .enabled_extension_names(&layers_names_raw);

        let device: erupt::DeviceLoader = unsafe {
            DeviceLoader::new(&instance, physical_device, &device_create_info, None).unwrap()
        };

        TestHarness {
            entry,
            instance: Arc::new(instance),
            device: Arc::new(device),
            physical_device,
            //debug_report_loader,
            //debug_callback,
        }
    }

    pub fn create_allocator(&self) -> vk_mem_erupt::Allocator {
        let create_info = vk_mem_erupt::AllocatorCreateInfo {
            physical_device: self.physical_device,
            device: Arc::clone(&self.device),
            instance: Arc::clone(&self.instance),
            flags: Default::default(),
            preferred_large_heap_block_size: 0,
            frame_in_use_count: 0,
            heap_size_limits: None,
        };
        vk_mem_erupt::Allocator::new(&create_info).unwrap()
    }
}

#[test]
fn create_harness() {
    let _ = TestHarness::new();
}

#[test]
fn create_allocator() {
    let harness = TestHarness::new();
    let _ = harness.create_allocator();
}

// #[test]
// fn default_allocator_create_info() {
//     //let _ = vk_mem_erupt::AllocatorCreateInfo;
// }

#[test]
fn create_gpu_buffer() {
    let harness = TestHarness::new();
    let allocator = harness.create_allocator();
    let allocation_info = vk_mem_erupt::AllocationCreateInfo {
        usage: vk_mem_erupt::MemoryUsage::GpuOnly,
        ..Default::default()
    };
    let (buffer, allocation, allocation_info) = allocator
        .create_buffer(
            &erupt::vk::BufferCreateInfoBuilder::new()
                .size(16 * 1024)
                .usage(
                    erupt::vk::BufferUsageFlags::VERTEX_BUFFER
                        | erupt::vk::BufferUsageFlags::TRANSFER_DST,
                )
                .build(),
            &allocation_info,
        )
        .unwrap();
    assert_eq!(allocation_info.get_mapped_data(), std::ptr::null_mut());
    allocator.destroy_buffer(buffer, &allocation);
}

#[test]
fn create_cpu_buffer_preferred() {
    let harness = TestHarness::new();
    let allocator = harness.create_allocator();
    let allocation_info = vk_mem_erupt::AllocationCreateInfo {
        required_flags: erupt::vk::MemoryPropertyFlags::HOST_VISIBLE,
        preferred_flags: erupt::vk::MemoryPropertyFlags::HOST_COHERENT
            | erupt::vk::MemoryPropertyFlags::HOST_CACHED,
        flags: vk_mem_erupt::AllocationCreateFlags::MAPPED,
        ..Default::default()
    };
    let (buffer, allocation, allocation_info) = allocator
        .create_buffer(
            &erupt::vk::BufferCreateInfoBuilder::new()
                .size(16 * 1024)
                .usage(
                    erupt::vk::BufferUsageFlags::VERTEX_BUFFER
                        | erupt::vk::BufferUsageFlags::TRANSFER_DST,
                )
                .build(),
            &allocation_info,
        )
        .unwrap();
    assert_ne!(allocation_info.get_mapped_data(), std::ptr::null_mut());
    allocator.destroy_buffer(buffer, &allocation);
}

#[test]
fn create_gpu_buffer_pool() {
    let harness = TestHarness::new();
    let allocator = harness.create_allocator();

    let buffer_info = erupt::vk::BufferCreateInfoBuilder::new()
        .size(16 * 1024)
        .usage(
            erupt::vk::BufferUsageFlags::UNIFORM_BUFFER | erupt::vk::BufferUsageFlags::TRANSFER_DST,
        )
        .build();

    let mut allocation_info = vk_mem_erupt::AllocationCreateInfo {
        required_flags: erupt::vk::MemoryPropertyFlags::HOST_VISIBLE,
        preferred_flags: erupt::vk::MemoryPropertyFlags::HOST_COHERENT
            | erupt::vk::MemoryPropertyFlags::HOST_CACHED,
        flags: vk_mem_erupt::AllocationCreateFlags::MAPPED,
        ..Default::default()
    };

    let memory_type_index = allocator
        .find_memory_type_index_for_buffer_info(&buffer_info, &allocation_info)
        .unwrap();

    // Create a pool that can have at most 2 blocks, 128 MiB each.
    let pool_info = vk_mem_erupt::AllocatorPoolCreateInfo {
        memory_type_index,
        block_size: 128 * 1024 * 1024,
        max_block_count: 2,
        ..Default::default()
    };
    let pool = allocator.create_pool(&pool_info).unwrap();
    allocation_info.pool = Some(pool.clone());

    let (buffer, allocation, allocation_info) = allocator
        .create_buffer(&buffer_info, &allocation_info)
        .unwrap();
    assert_ne!(allocation_info.get_mapped_data(), std::ptr::null_mut());
    allocator.destroy_buffer(buffer, &allocation);
    allocator.destroy_pool(&pool);
}

#[test]
fn test_gpu_stats() {
    let harness = TestHarness::new();
    let allocator = harness.create_allocator();
    let allocation_info = vk_mem_erupt::AllocationCreateInfo {
        usage: vk_mem_erupt::MemoryUsage::GpuOnly,
        ..Default::default()
    };

    let stats_1 = allocator.calculate_stats().unwrap();
    assert_eq!(stats_1.total.blockCount, 0);
    assert_eq!(stats_1.total.allocationCount, 0);
    assert_eq!(stats_1.total.usedBytes, 0);

    let (buffer, allocation, _allocation_info) = allocator
        .create_buffer(
            &erupt::vk::BufferCreateInfoBuilder::new()
                .size(16 * 1024)
                .usage(
                    erupt::vk::BufferUsageFlags::VERTEX_BUFFER
                        | erupt::vk::BufferUsageFlags::TRANSFER_DST,
                )
                .build(),
            &allocation_info,
        )
        .unwrap();

    let stats_2 = allocator.calculate_stats().unwrap();
    assert_eq!(stats_2.total.blockCount, 1);
    assert_eq!(stats_2.total.allocationCount, 1);
    assert_eq!(stats_2.total.usedBytes, 16 * 1024);

    allocator.destroy_buffer(buffer, &allocation);

    let stats_3 = allocator.calculate_stats().unwrap();
    assert_eq!(stats_3.total.blockCount, 1);
    assert_eq!(stats_3.total.allocationCount, 0);
    assert_eq!(stats_3.total.usedBytes, 0);
}

#[test]
fn test_stats_string() {
    let harness = TestHarness::new();
    let allocator = harness.create_allocator();
    let allocation_info = vk_mem_erupt::AllocationCreateInfo {
        usage: vk_mem_erupt::MemoryUsage::GpuOnly,
        ..Default::default()
    };

    let stats_1 = allocator.build_stats_string(true).unwrap();
    assert!(stats_1.len() > 0);

    let (buffer, allocation, _allocation_info) = allocator
        .create_buffer(
            &erupt::vk::BufferCreateInfoBuilder::new()
                .size(16 * 1024)
                .usage(
                    erupt::vk::BufferUsageFlags::VERTEX_BUFFER
                        | erupt::vk::BufferUsageFlags::TRANSFER_DST,
                )
                .build(),
            &allocation_info,
        )
        .unwrap();

    let stats_2 = allocator.build_stats_string(true).unwrap();
    assert!(stats_2.len() > 0);
    assert_ne!(stats_1, stats_2);

    allocator.destroy_buffer(buffer, &allocation);

    let stats_3 = allocator.build_stats_string(true).unwrap();
    assert!(stats_3.len() > 0);
    assert_ne!(stats_3, stats_1);
    assert_ne!(stats_3, stats_2);
}
