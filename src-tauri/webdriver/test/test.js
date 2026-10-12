const path = require("path");
const { spawn, spawnSync } = require("child_process");
const { existsSync } = require("fs");
const { Builder, By, Capabilities, until } = require("selenium-webdriver");
const { expect } = require("chai");

const ROOT_DIR = path.resolve(__dirname, "..", "..", "..");
const MOCK_ENV = {
  ...process.env,
  NEXT_PUBLIC_TAURI_WEBDRIVER_MOCKS: "1",
  TZ: "UTC",
};
const APPLICATION_NAME = process.platform === "win32" ? "app.exe" : "app";
const APPLICATION_TARGET_DIR = path.resolve(ROOT_DIR, "src-tauri", "target");
const DRIVER_WAIT = 15_000;
const TAURI_DRIVER_COMMAND = process.env.TAURI_DRIVER_PATH || "tauri-driver";

let driver;
let tauriDriver;
let builtApplicationPath;

before(async function () {
  this.timeout(300000);

  const build = spawnSync(
    "bun",
    ["--bun", "run", "tauri", "build", "--no-bundle"],
    {
      cwd: ROOT_DIR,
      env: MOCK_ENV,
      stdio: "inherit",
      shell: process.platform === "win32",
    }
  );

  if (build.error) {
    throw build.error;
  }
  if (build.status !== 0) {
    throw new Error("bun --bun run tauri build failed");
  }

  const releaseCandidate = path.resolve(
    APPLICATION_TARGET_DIR,
    "release",
    APPLICATION_NAME
  );
  const debugCandidate = path.resolve(
    APPLICATION_TARGET_DIR,
    "debug",
    APPLICATION_NAME
  );
  builtApplicationPath =
    [releaseCandidate, debugCandidate].find((candidate) => existsSync(candidate)) ??
    null;

  if (!builtApplicationPath) {
    throw new Error(
      `Built application not found at either ${releaseCandidate} or ${debugCandidate}. Run bun --bun run tauri build manually.`
    );
  }

  tauriDriver = spawn(TAURI_DRIVER_COMMAND, [], {
    env: process.env,
    stdio: ["ignore", process.stdout, process.stderr],
  });

  await new Promise((resolve) => setTimeout(resolve, 1000));

  const capabilities = new Capabilities();
  capabilities.set("tauri:options", { application: builtApplicationPath });
  capabilities.setBrowserName("wry");

  driver = await new Builder()
    .withCapabilities(capabilities)
    .usingServer("http://localhost:4444/")
    .build();
});

after(async function () {
  await closeTauriDriver();
});

describe("Dashboard workflow", function () {
  this.timeout(DRIVER_WAIT * 2);
  it("shows the connection status and navigation controls", async function () {
    const statusElement = await driver.wait(
      until.elementLocated(By.css("header p")),
      DRIVER_WAIT
    );
    await driver.wait(
      until.elementTextContains(statusElement, "Mock connected to gRPC server"),
      DRIVER_WAIT
    );

    const settingsLink = await driver.findElement(By.css('a[href="/settings"]'));
    const homeLink = await driver.findElement(By.css('a[href="/"]'));

    expect(await settingsLink.isDisplayed()).to.be.true;
    expect(await homeLink.isDisplayed()).to.be.true;
  });

  it("can toggle the proxy fields and submit updated settings", async function () {
    await driver.executeScript('window.location.pathname = "/settings"');

    const urlInput = await driver.wait(
      until.elementLocated(
        By.css('input[placeholder="https://example.com/grpc"]')
      ),
      DRIVER_WAIT
    );
    const accessTokenInput = await driver.findElement(
      By.css('input[placeholder="Your Access Token"]')
    );

    await accessTokenInput.clear();
    await accessTokenInput.sendKeys("another-token");

    const proxyCheckbox = await driver.findElement(By.css("#use-proxies"));
    const proxyInput = await driver.findElement(
      By.css('input[placeholder="http://proxy.example.com:8080"]')
    );

    expect(await proxyInput.getAttribute("disabled")).to.be.ok;
    await proxyCheckbox.click();

    await driver.wait(async () => {
      return (await proxyInput.getAttribute("disabled")) === null;
    }, DRIVER_WAIT);

    await proxyInput.clear();
    await proxyInput.sendKeys("http://proxy.local:8080");

    const updateButton = await driver.findElement(
      By.xpath('//button[normalize-space()="Update"]')
    );
    await updateButton.click();
  });

  it("shows secure-storage load and save errors without reconnecting", async function () {
    await driver.executeScript('sessionStorage.setItem("roomtemp.mock.settings.load", "fail"); window.location.pathname = "/settings"');
    await driver.wait(until.elementLocated(By.css('[role="alert"]')), DRIVER_WAIT);
    expect(await driver.findElement(By.css('[role="alert"]')).getText()).to.include("Keystore: storage unavailable");
    await driver.executeScript('sessionStorage.removeItem("roomtemp.mock.settings.load"); window.location.reload()');
    const token = await driver.wait(until.elementLocated(By.css('#access-token')), DRIVER_WAIT);
    await driver.wait(async () => (await token.getAttribute("value")) === "test-token", DRIVER_WAIT);
    const connections = await driver.executeScript('return sessionStorage.getItem("roomtemp.mock.connections")');
    await driver.executeScript('sessionStorage.setItem("roomtemp.mock.settings.save", "fail")');
    await driver.findElement(By.xpath('//button[normalize-space()="Update"]')).click();
    await driver.wait(until.elementLocated(By.css('[role="alert"]')), DRIVER_WAIT);
    expect(await driver.findElement(By.css('[role="alert"]')).getText()).to.include("Keystore: storage unavailable");
    expect(await driver.executeScript('return sessionStorage.getItem("roomtemp.mock.connections")')).to.equal(connections);
    await driver.executeScript('sessionStorage.removeItem("roomtemp.mock.settings.save")');
    await driver.findElement(By.xpath('//button[normalize-space()="Update"]')).click();
    await driver.wait(async () => Number(await driver.executeScript('return sessionStorage.getItem("roomtemp.mock.connections")')) > Number(connections), DRIVER_WAIT);
    expect((await driver.findElements(By.css('[role="alert"]'))).length).to.equal(0);
  });

  it("renders the charts after fetching graph data", async function () {
    await driver.executeScript('window.location.pathname = "/"');

    await driver.wait(
      until.elementLocated(By.xpath('//button[normalize-space()="Fetch Data"]')),
      DRIVER_WAIT
    );
    await driver.executeScript(`
      const iterator = document.evaluate(
        '//button[normalize-space()="Fetch Data"]',
        document,
        null,
        XPathResult.FIRST_ORDERED_NODE_TYPE,
        null
      );
      const button = iterator.singleNodeValue;
      if (!button) throw new Error("Fetch Data button not found");
      button.click();
    `);

    const legends = [
      "Temperature (℃)",
      "Humidity (%)",
      "Illumination (lx)",
    ];

    for (const label of legends) {
      const found = await driver.wait(
        async () => {
          const text = await driver.executeScript(
            "return document.body.innerText || '';"
          );
          return text.includes(label);
        },
        DRIVER_WAIT
      );
      expect(found).to.be.true;
    }
  });
});

async function closeTauriDriver() {
  if (driver) {
    await driver.quit();
    driver = null;
  }
  if (tauriDriver) {
    tauriDriver.kill();
    tauriDriver = null;
  }
}

function onShutdown(fn) {
  const cleanup = () => {
    try {
      fn();
    } finally {
      process.exit();
    }
  };

  process.on("exit", cleanup);
  process.on("SIGINT", cleanup);
  process.on("SIGTERM", cleanup);
  process.on("SIGHUP", cleanup);
  process.on("SIGBREAK", cleanup);
}

onShutdown(async () => {
  await closeTauriDriver();
});
