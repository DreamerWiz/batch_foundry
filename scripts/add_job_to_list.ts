
import Redis from "ioredis";
import fs from "fs";
import path from "path";


const redis = new Redis({
  host: "localhost",
  port: 6379
});

type PathWithContent = {
  path: string;
  content: string;
}

type Job = {
  questionNo: string;
  pathWithContent: PathWithContent[];
}

async function delay(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}


function formatDate(date: Date): string {
  const year = date.getFullYear();
  const month = date.getMonth() + 1; // getMonth() 返回的月份是从0开始的
  const day = date.getDate();

  // 将月份和日期格式化为两位数
  const formattedMonth = month < 10 ? `0${month}` : `${month}`;
  const formattedDay = day < 10 ? `0${day}` : `${day}`;

  return `${year}${formattedMonth}${formattedDay}`;
}

function generateRandomString(length: number): string {
  const characters = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
  let result = '';
  const charactersLength = characters.length;
  for (let i = 0; i < length; i++) {
      result += characters.charAt(Math.floor(Math.random() * charactersLength));
  }
  return result;
}

function generatePrefixedRandomString(prefixLength: number, randomStringLength: number): string {
  const datePrefix = formatDate(new Date());
  const randomString = generateRandomString(randomStringLength);
  return `${datePrefix}${randomString}`;
}
async function main(){
   
  const dirents = fs.readdirSync("contracts");

  let counter = 0;
  for(const dirent of dirents){
    const p = path.join("contracts", dirent);
    const res = fs.statSync(p);
    if(res.isDirectory()){
      const basePath = p;
      const sn = dirent;
      const pathWithContentList:PathWithContent[] = [];

      let contractsFiles = fs.readdirSync(path.join(basePath, "contracts"));

      contractsFiles.filter((f)=> {
        return f.endsWith(".ans")
      }).forEach((f) => { 
        pathWithContentList.push({
          path: path.join("contracts", f.substring(0, f.length - 4)),
          content: fs.readFileSync(path.join(basePath, "contracts", f)).toString()
        })
      })

      contractsFiles.filter((f)=> {
        return f.endsWith(".ans") == false
      }).forEach((f) => { 
        pathWithContentList.push({
          path: path.join("contracts", f),
          content: fs.readFileSync(path.join(basePath, "contracts", f)).toString()
        })
      })

      let testFiles = fs.readdirSync(path.join(basePath, "test"));
      testFiles.forEach((t) => {
        pathWithContentList.push({
          path: path.join("test", t),
          content: fs.readFileSync(path.join(basePath, "test", t)).toString()
        })
      })

     let judgeJobId = generatePrefixedRandomString(8, 10);
      const jobMessage = {
        questionNo: sn,
        pathWithContent: pathWithContentList,
        solcVersion: "0.8.20",
        judgeJobId: judgeJobId
      }

      sendJobWithTimeout(jobMessage).then(result => console.log("Result: " + result)).catch(error => console.log("Error: "+ error.message));
    }
    break;
  }
}

function withTimeout<T>(promise: Promise<T>, timeoutMs: number, timeoutValue: T): Promise<T> {
  // 创建一个超时 Promise
  let timeoutPromise = new Promise<T>((resolve) => {
    let id = setTimeout(() => {
      clearTimeout(id);
      resolve(timeoutValue);
    }, timeoutMs);
  });

  // 返回最先解决的 Promise：原始 Promise 或超时 Promise
  return Promise.race([promise, timeoutPromise]);
}

async function sendJob(params: any) {
  return new Promise(async (resolve) => {
    let res = await redis.get("test:test-box-job-response:" + params.judgeJobId);
  })
}


async function sendJobWithTimeout(params: any) {
  params.jobKey = "test:test-box-job:" + params.judgeJobId;
  await redis.set(params.jobKey + ":request", "");
  await redis.lpush("test", JSON.stringify(params));

  async function getResponse(){
    return new Promise(async (resolve, _) => {
      for(let i =0; i< 15;i++){
        const res = await redis.get(params.jobKey + ":response");
        await new Promise(() => setTimeout(() => {}, 100));
        console.log(("res:" + res));
        if(res){
          return res;
        }
      }
    })
  }

  const timeOut = new Promise((_, reject) => setTimeout(() => reject(new Error('Timeout')), 2000));

  return Promise.race([getResponse(), timeOut]);
}

main();