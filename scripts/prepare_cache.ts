import {execSync} from 'child_process';
import fs from "fs";


// let template = String(fs.readFileSync("cache/exmple-cache.json"));

// let templateJson = JSON.parse(template) as any;

// console.log(templateJson.files);

// const files = templateJson.files;

// const filesKeys = Object.keys(files).filter((obj) => obj.startsWith("lib/"));

// console.log(filesKeys);

// const fileKeysAuto = [
//   'lib/forge-std/lib/ds-test/src/test.sol',
//   'lib/forge-std/src/Base.sol',
//   'lib/forge-std/src/StdAssertions.sol',
//   'lib/forge-std/src/StdChains.sol',
//   'lib/forge-std/src/StdCheats.sol',
//   'lib/forge-std/src/StdError.sol',
//   'lib/forge-std/src/StdInvariant.sol',
//   'lib/forge-std/src/StdJson.sol',
//   'lib/forge-std/src/StdMath.sol',
//   'lib/forge-std/src/StdStorage.sol',
//   'lib/forge-std/src/StdStyle.sol',
//   'lib/forge-std/src/StdUtils.sol',
//   'lib/forge-std/src/Test.sol',
//   'lib/forge-std/src/Vm.sol',
//   'lib/forge-std/src/console.sol',
//   'lib/forge-std/src/console2.sol',
//   'lib/forge-std/src/interfaces/IMulticall3.sol',
//   'lib/forge-std/src/safeconsole.sol'
// ];



(async function(){
  const version_list = ["0.8.20", "0.8.21", "0.8.22", "0.8.23"]

  execSync("rm -rf out && mkdir out");
  
  let res, version = version_list[0];
  res = await execSync("forge build --contracts lib/forge-std/src --out tmp_out --use " + version);
  console.log(String(res));
  // res =await execSync("forge build --contracts lib/openzeppelin-contracts/contracts --out tmp_out --use " + version);
  // console.log(String(res));

  const initObj = JSON.parse(String(fs.readFileSync("cache/solidity-files-cache.json")));

  initObj.paths.sources = "[fixme]";

  for( const fName of Object.keys(initObj.files)){
    const artifacts = initObj.files[fName].artifacts;
    for(const cName of Object.keys(artifacts)){
      for(const [compiler, location] of Object.entries(artifacts[cName])){
        initObj.files[fName].artifacts[cName][compiler] = version + "/" + location;
      }
    }
  }

  execSync("cp -r tmp_out" + " out/" + version);

  for( const v of version_list.slice(1, version_list.length - 1)){
    res = await execSync("forge build --contracts lib/forge-std/src --out tmp_out --use " + v);
    console.log(String(res));
    // res =await execSync("forge build --contracts lib/openzeppelin-contracts/contracts --out tmp_out --use " + v);
    // console.log(String(res));

    const obj = JSON.parse(String(fs.readFileSync("cache/solidity-files-cache.json")));


    for( const fName of Object.keys(obj.files)){
      const artifacts = obj.files[fName].artifacts;
      for(const cName of Object.keys(artifacts)){
        for(const [compiler, location] of Object.entries(artifacts[cName])){
          initObj.files[fName].artifacts[cName][compiler] = v + "/" + location;
        }
      }
    }

    execSync("cp -r tmp_out" + " out/" + v);
    execSync("rm -rf tmp_out");
  }

  fs.writeFileSync("cache/example-cache.json", JSON.stringify(initObj));
})()