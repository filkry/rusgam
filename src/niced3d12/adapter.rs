use super::*;

pub struct SAdapter {
    raw: t12::SAdapter4,
}

impl SAdapter {
    pub fn new_from_raw(raw: t12::SAdapter4) -> Self {
        Self { raw: raw }
    }

    pub fn create_device(&mut self, d3d_debug: bool) -> Result<SDevice, &'static str> {
        let device = unsafe { self.raw.d3d12createdevice()? };

        if d3d_debug {
            match device.castinfoqueue() {
                Some(infoqueue) => {
                    infoqueue.set_break_on_severity(win::D3D12_MESSAGE_SEVERITY_CORRUPTION, true);
                    infoqueue.set_break_on_severity(win::D3D12_MESSAGE_SEVERITY_ERROR, true);
                    infoqueue.set_break_on_severity(win::D3D12_MESSAGE_SEVERITY_WARNING, true);

                    let mut suppressedseverities = [win::D3D12_MESSAGE_SEVERITY_INFO];

                    let mut suppressedmessages =
                        [win::D3D12_MESSAGE_ID_CLEARRENDERTARGETVIEW_MISMATCHINGCLEARVALUE];

                    let allowlist = win::D3D12_INFO_QUEUE_FILTER_DESC {
                        NumCategories: 0,
                        pCategoryList: ptr::null_mut(),
                        NumSeverities: 0,
                        pSeverityList: ptr::null_mut(),
                        NumIDs: 0,
                        pIDList: ptr::null_mut(),
                    };

                    let denylist = win::D3D12_INFO_QUEUE_FILTER_DESC {
                        NumCategories: 0,
                        pCategoryList: ptr::null_mut(),
                        NumSeverities: suppressedseverities.len() as u32,
                        pSeverityList: &mut suppressedseverities[0],
                        NumIDs: suppressedmessages.len() as u32,
                        pIDList: &mut suppressedmessages[0],
                    };

                    let mut filter = win::D3D12_INFO_QUEUE_FILTER {
                        AllowList: allowlist,
                        DenyList: denylist,
                    };

                    match infoqueue.pushstoragefilter(&mut filter) {
                        Ok(_) => (),
                        Err(e) => return Err(e),
                    }
                }
                None => {
                    return Err("Could not get info queue from adapter.");
                }
            }
        }

        Ok(SDevice::new_from_raw(device))
    }
}
