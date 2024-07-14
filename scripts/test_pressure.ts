import { execSync } from 'child_process';
import fs from 'fs';
import path from "path";

import { workerData } from 'worker_threads';

function randomChoice<T>(items: T[]): T {
  const index = Math.floor(Math.random() * items.length);
  return items[index];
}

function logRed(text: string) {
  // 使用 ANSI 转义序列来设置文本颜色为红色
  const RED = "\x1b[31m";
  // 重置颜色回默认
  const RESET = "\x1b[0m";

  console.log(`${RED}${text}${RESET}`);
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

// 使用函

(async function () {
  let dirs = fs.readdirSync("contracts");


  while (true) {
    const start = Date.now();
    const temp_dir = process.argv[2];

    let dir = randomChoice(dirs);

    let contracts = fs.readdirSync(path.join("contracts", dir, "contracts")).filter(x => {
      return x.endsWith("ans");
    });

    const judgeJobId = generatePrefixedRandomString(8, 10)

    const random_ans = randomChoice(contracts);

    // console.log(dir, random_ans);

    // console.log("cp " + path.join("contracts", dir, "contracts", random_ans) + " " + path.join(temp_dir, "contracts/", random_ans.substring(0, random_ans.length - 6)));

    execSync("cp -r " + path.join("contracts", dir) + " ./" + temp_dir);

    execSync("cp " + path.join("contracts", dir, "contracts", random_ans) + " " + path.join(temp_dir, "contracts/", random_ans.substring(0, random_ans.length - 6)));

    // // console.log(contracts);

    // console.log("docker run -t --rm --network host judger smc-open-solc-judger client -n " + dir + " -d " + temp_dir + " --job-id " + judgeJobId + " --timeout 5");

    const cmd = "docker run -t --rm --network host -v ./" + temp_dir + ":/app/usercode batch-foundry client -n " + dir + " --job-id " + judgeJobId + " --timeout 5";
    console.log(cmd);
    let res = execSync(cmd);

    // console.log(new String(res));

    execSync("rm -rf " + temp_dir);
    // console.log(String(res));
    const json = JSON.parse(String(res));

    const endTime = Date.now();

    console.log(json);
    if (json["info"] == "Timeout") {
      logRed(dir + " ans[" + random_ans[random_ans.length - 5] + "]" + " " + judgeJobId + " " + json["costTime"] + " " + json["info"] + " scriptTime("+(endTime - start)/1000.0+"s)");
    } else {
      console.log(dir + " ans[" + random_ans[random_ans.length - 5] + "]" + " " + judgeJobId + " " + json["costTime"] + " " + json["info"] + " scriptTime("+(endTime - start)/1000.0+"s)");
    }
  }
})()