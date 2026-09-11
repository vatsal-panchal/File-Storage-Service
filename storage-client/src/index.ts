import * as fs from "node:fs";
import * as path from "node:path";
import { FileStorageClient } from "./client.js";

async function run() {
  const client = new FileStorageClient("http://127.0.0.1:3000");

  try {
    const health = await client.health();
    console.log("Server Health:", health);

    const testDir = path.resolve("./temp");
    if (!fs.existsSync(testDir)) {
      fs.mkdirSync(testDir, { recursive: true });
    }

    const testFilePath = path.join(testDir, "sample.txt");
    fs.writeFileSync(testFilePath, "Hello from File Storage Service! Testing Rust & TypeScript integration.");

    console.log("\nUploading file:", testFilePath);
    const uploaded = await client.uploadFile(testFilePath);
    console.log("Uploaded successfully:", uploaded);

    console.log("\nListing files in storage:");
    const files = await client.listFiles();
    console.log(files);

    console.log("\nChecking file existence for ID:", uploaded.id);
    const existsBefore = await client.checkFileExists(uploaded.id);
    console.log("File exists:", existsBefore);

    console.log("\nSearching files with query 'sample' and extension 'txt':");
    const matchedFiles = await client.listFiles({ query: "sample", extension: "txt" });
    console.log("Matched files count:", matchedFiles.length);

    console.log("\nFetching file info for ID:", uploaded.id);
    const info = await client.getFileInfo(uploaded.id);
    console.log(info);

    const downloadPath = path.join(testDir, "downloaded_sample.txt");
    console.log("\nDownloading file to:", downloadPath);
    await client.downloadFile(uploaded.id, downloadPath);
    const downloadedContent = fs.readFileSync(downloadPath, "utf-8");
    console.log("Downloaded content matches:", downloadedContent === fs.readFileSync(testFilePath, "utf-8"));

    console.log("\nDeleting file with ID:", uploaded.id);
    const deleteResult = await client.deleteFile(uploaded.id);
    console.log(deleteResult);

    console.log("\nFinal files list:");
    const finalFiles = await client.listFiles();
    console.log(finalFiles);

    fs.rmSync(testDir, { recursive: true, force: true });
    console.log("\nDemo completed successfully.");
  } catch (err) {
    console.error("Execution error:", err);
  }
}

run();
