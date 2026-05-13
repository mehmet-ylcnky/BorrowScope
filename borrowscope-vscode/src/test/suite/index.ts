import * as path from "path";

export function run(): Promise<void> {
  // eslint-disable-next-line @typescript-eslint/no-var-requires
  const Mocha = require("mocha");
  const glob = require("glob");

  const mocha = new Mocha({ ui: "tdd", timeout: 60000 });
  const testsRoot = path.resolve(__dirname);

  return new Promise((resolve, reject) => {
    glob("**/integration.test.js", { cwd: testsRoot }, (err: any, files: string[]) => {
      if (err) return reject(err);
      files.forEach((f: string) => mocha.addFile(path.resolve(testsRoot, f)));
      mocha.run((failures: number) => {
        if (failures > 0) reject(new Error(`${failures} tests failed`));
        else resolve();
      });
    });
  });
}
