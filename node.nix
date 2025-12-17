{ pkgs, system }:

let
  # Determine the appropriate Node.js binary based on system
  nodeBinaryInfo =
    if system == "x86_64-linux" then
      {
        url = "https://nodejs.org/dist/v22.18.0/node-v22.18.0-linux-x64.tar.xz";
        sha256 = "c1bfeecf1d7404fa74728f9db72e697decbd8119ccc6f5a294d795756dfcfca7";
      }
    else if system == "aarch64-linux" then
      {
        url = "https://nodejs.org/dist/v22.18.0/node-v22.18.0-linux-arm64.tar.xz";
        sha256 = "04fca1b9afecf375f26b41d65d52aa1703a621abea5a8948c7d1e351e85edade";
      }
    else if system == "x86_64-darwin" then
      {
        url = "https://nodejs.org/dist/v22.18.0/node-v22.18.0-darwin-x64.tar.xz";
        sha256 = "76e4a1997da953dbf8e21f6ed1c4dd7eceb39deb96defe3b3e9d8f786ee287a8";
      }
    else if system == "aarch64-darwin" then
      {
        url = "https://nodejs.org/dist/v22.18.0/node-v22.18.0-darwin-arm64.tar.xz";
        sha256 = "6616f388e127c858989fc7fa92879cdb20d2a5d446adbfdca6ee4feb385bfa8a";
      }
    else
      abort "Unsupported system: ${system}. Please update the node binary info for your system in the flake definition manually.";
in
# Create a Node.js package from the pre-built binary
pkgs.stdenv.mkDerivation {
  pname = "nodejs-binary";
  version = "22.18.0";

  src = pkgs.fetchurl {
    url = nodeBinaryInfo.url;
    sha256 = nodeBinaryInfo.sha256;
  };

  installPhase = ''
    mkdir -p $out
    cp -r * $out/
  '';

  dontBuild = true;
  dontConfigure = true;
}
