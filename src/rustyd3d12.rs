

impl SFactory {
    pub fn bestadapter(&self) -> Result<SAdapter, &'static str> {
        //let mut rawadapter4: *mut IDXGIFactory4 = ptr::null_mut();
        let mut maxdedicatedmem: usize = 0;
        let mut bestadapter = 0;

        for adapteridx in 0..10 {
            let mut adapter1 = self.enumadapters(adapteridx);
            let adapterdesc = adapter1.getdesc();

            if adapterdesc.Flags & DXGI_ADAPTER_FLAG_SOFTWARE > 0 {
                continue;
            }

            let devicecreateresult = adapter1.d3d12createdevice();
            if !winerror::SUCCEEDED(devicecreateresult) {
                continue;
            }

            if adapterdesc.DedicatedVideoMemory > maxdedicatedmem {
                match adapter1.castadapter4() {
                    Ok(_) => {
                        bestadapter = adapteridx;
                        maxdedicatedmem = adapterdesc.DedicatedVideoMemory;
                    }
                    Err(_) => {}
                }
            }
        }

        if maxdedicatedmem > 0 {
            let adapter1 = self.enumadapters(bestadapter)
            match adapter1.castadapter4() {
                Ok(a) => {
                    return Ok(SAdapter { adapter: a });
                }
                Err(_) => {
                    return Err("Getting Adapter4 failed despite working earlier");
                }
            };
        }

        Err("Could not find valid adapter")
    }
}

impl SAdapter {
    pub fn createdevice(&mut self) -> Result<SDevice, &'static str> {
        // -- $$$FRK(TODO): remove unwraps
        let device = self.createdevice().unwrap();

        // -- $$$FRK(TODO): debug only
        match device.castcastinfoqueue() {
            Ok(infoqueue) => {
                infoqueue.SetBreakOnSeverity(D3D12_MESSAGE_SEVERITY_CORRUPTION, TRUE);
                infoqueue.SetBreakOnSeverity(D3D12_MESSAGE_SEVERITY_ERROR, TRUE);
                infoqueue.SetBreakOnSeverity(D3D12_MESSAGE_SEVERITY_WARNING, TRUE);

                let mut suppressedseverities = [D3D12_MESSAGE_SEVERITY_INFO];

                let mut suppressedmessages =
                    [D3D12_MESSAGE_ID_CLEARRENDERTARGETVIEW_MISMATCHINGCLEARVALUE];

                let allowlist = D3D12_INFO_QUEUE_FILTER_DESC {
                    NumCategories: 0,
                    pCategoryList: ptr::null_mut(),
                    NumSeverities: 0,
                    pSeverityList: ptr::null_mut(),
                    NumIDs: 0,
                    pIDList: ptr::null_mut(),
                };

                let denylist = D3D12_INFO_QUEUE_FILTER_DESC {
                    NumCategories: 0,
                    pCategoryList: ptr::null_mut(),
                    NumSeverities: suppressedseverities.len() as u32,
                    pSeverityList: &mut suppressedseverities[0] as *mut u32,
                    NumIDs: suppressedmessages.len() as u32,
                    pIDList: &mut suppressedmessages[0] as *mut u32,
                };

                let mut filter = D3D12_INFO_QUEUE_FILTER {
                    AllowList: allowlist,
                    DenyList: denylist,
                };

                match infoqueue.PushStorageFilter(&mut filter)  {
                    Ok() => (),
                    Err(e) => return Err(e),
                }
            }
            Err(_) => {
                return Err("Could not get info queue from adapter.");
            }
        }

        Ok(SDevice { device: device })
    }
}