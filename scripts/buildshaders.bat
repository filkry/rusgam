CALL "externals/dxc_2019-07-15/dxc.exe" -E main -T ps_6_0 "shaders/pixel.hlsl" -Fo "shaders_built/pixel.cso"
CALL "externals/dxc_2019-07-15/dxc.exe" -E main -T vs_6_0 "shaders/vertex.hlsl" -Fo "shaders_built/vertex.cso"
