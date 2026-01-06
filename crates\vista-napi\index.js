const { existsSync, readFileSync } = require('fs');
const { join } = require('path');
const { platform, arch } = process;

const NATIVE_BINDINGS = {
  android: {
    'arm64': ['vista-native.android-arm64.node', 'vista-native-android-arm64'],
    'arm': ['vista-native.android-arm-eabi.node', 'vista-native-android-arm-eabi'],
  },
  win32: {
    'x64': ['vista-native.win32-x64-msvc.node', 'vista-native-win32-x64-msvc'],
    'ia32': ['vista-native.win32-ia32-msvc.node', 'vista-native-win32-ia32-msvc'],
    'arm64': ['vista-native.win32-arm64-msvc.node', 'vista-native-win32-arm64-msvc'],
  },
  darwin: {
    'x64': ['vista-native.darwin-x64.node', 'vista-native-darwin-x64'],
    'arm64': ['vista-native.darwin-arm64.node', 'vista-native-darwin-arm64'],
    universal: ['vista-native.darwin-universal.node', 'vista-native-darwin-universal'],
  },
  freebsd: {
    'x64': ['vista-native.freebsd-x64.node', 'vista-native-freebsd-x64'],
  },
  linux: {
    'x64': {
      musl: ['vista-native.linux-x64-musl.node', 'vista-native-linux-x64-musl'],
      gnu: ['vista-native.linux-x64-gnu.node', 'vista-native-linux-x64-gnu'],
    },
    'arm64': {
      musl: ['vista-native.linux-arm64-musl.node', 'vista-native-linux-arm64-musl'],
      gnu: ['vista-native.linux-arm64-gnu.node', 'vista-native-linux-arm64-gnu'],
    },
    'arm': {
      musl: ['vista-native.linux-arm-musleabihf.node', 'vista-native-linux-arm-musleabihf'],
      gnu: ['vista-native.linux-arm-gnueabihf.node', 'vista-native-linux-arm-gnueabihf'],
    },
    'riscv64': {
      musl: ['vista-native.linux-riscv64-musl.node', 'vista-native-linux-riscv64-musl'],
      gnu: ['vista-native.linux-riscv64-gnu.node', 'vista-native-linux-riscv64-gnu'],
    },
    's390x': ['vista-native.linux-s390x-gnu.node', 'vista-native-linux-s390x-gnu'],
  },
};

function isMusl() {
  if (!process.report || typeof process.report.getReport !== 'function') {
    try {
      const lddPath = require('child_process').execSync('which ldd').toString().trim();
      return readFileSync(lddPath, 'utf8').includes('musl');
    } catch {
      return true;
    }
  }
  const { glibcVersionRuntime } = process.report.getReport().header;
  return !glibcVersionRuntime;
}

let nativeBinding = null;
let loadError = null;

const getNativeBinding = () => {
  const platformKey = platform;
  const archKey = arch;

  if (!NATIVE_BINDINGS[platformKey]) {
    throw new Error(`Unsupported OS: ${platformKey}`);
  }

  let binding;
  if (platformKey === 'linux') {
    if (isMusl()) {
      binding = NATIVE_BINDINGS[platformKey][archKey].musl;
    } else {
      binding = NATIVE_BINDINGS[platformKey][archKey].gnu;
    }
  } else if (platformKey === 'darwin') {
    if (archKey === 'arm64' || archKey === 'x64') {
      binding = NATIVE_BINDINGS[platformKey][archKey];
    } else {
      binding = NATIVE_BINDINGS[platformKey].universal;
    }
  } else {
    binding = NATIVE_BINDINGS[platformKey][archKey];
  }

  if (Array.isArray(binding)) {
    const [localFile, packageName] = binding;
    const localFilePath = join(__dirname, localFile);
    if (existsSync(localFilePath)) {
      try {
        nativeBinding = require(localFilePath);
      } catch (e) {
        loadError = e;
      }
    } else {
      try {
        nativeBinding = require(packageName);
      } catch (e) {
        loadError = e;
      }
    }
  } else {
    const [localFile, packageName] = binding;
    const localFilePath = join(__dirname, localFile);
    if (existsSync(localFilePath)) {
      try {
        nativeBinding = require(localFilePath);
      } catch (e) {
        loadError = e;
      }
    } else {
      try {
        nativeBinding = require(packageName);
      } catch (e) {
        loadError = e;
      }
    }
  }
};

getNativeBinding();

if (!nativeBinding) {
  if (loadError) {
    throw loadError;
  }
  throw new Error('Failed to load native binding');
}

const {
  isClientComponent,
  analyzeClientDirective,
  getRouteTree,
  version,
  hasMetadataExport,
  hasGenerateMetadata,
  analyzeMetadata,
  rscScanApp,
  rscGenerateClientManifest,
  rscGenerateServerManifest,
  rscGenerateMountId,
  rscResetMountCounter,
  rscPrerenderComponent,
  rscPrerenderAllComponents,
} = nativeBinding;

module.exports.isClientComponent = isClientComponent;
module.exports.analyzeClientDirective = analyzeClientDirective;
module.exports.getRouteTree = getRouteTree;
module.exports.version = version;
module.exports.hasMetadataExport = hasMetadataExport;
module.exports.hasGenerateMetadata = hasGenerateMetadata;
module.exports.analyzeMetadata = analyzeMetadata;
module.exports.rscScanApp = rscScanApp;
module.exports.rscGenerateClientManifest = rscGenerateClientManifest;
module.exports.rscGenerateServerManifest = rscGenerateServerManifest;
module.exports.rscGenerateMountId = rscGenerateMountId;
module.exports.rscResetMountCounter = rscResetMountCounter;
module.exports.rscPrerenderComponent = rscPrerenderComponent;
module.exports.rscPrerenderAllComponents = rscPrerenderAllComponents;