import { getPreferenceValues, environment } from "@raycast/api";
import path from "path";
import os from "os";
import { execSync } from "child_process";
import fs from "fs";
import { createHash } from "crypto";
import { promisify } from "util";
import https from "https";
import { createWriteStream } from "fs";
import { pipeline } from "stream";
import { chmod, mkdir, rm } from "fs/promises";
// No need for exec since we're not extracting archives

export interface Preferences {
  geminiApiKey: string;
  geminiModelName: string;
  vocabularyBaseDir: string;
  gitEnabled: boolean;
  gitRemoteUrl: string;
}

// Get the default vocabulary path from the CLI's configuration
export function getVocabularyPath(): string {
  // Use a fixed path to avoid issues with CLI path resolution
  return path.join(os.homedir(), "word4you", "vocabulary_notebook.md");
}

// Ensure directory exists for vocabulary file
export function ensureVocabularyDirectoryExists(vocabularyPath: string): void {
  try {
    const dir = path.dirname(vocabularyPath);
    if (!fs.existsSync(dir)) {
      fs.mkdirSync(dir, { recursive: true });
    }
  } catch (err) {
    console.error("Error creating vocabulary directory:", err);
  }
}

// Check if word4you CLI is installed
export function isCliInstalled(): boolean {
  try {
    // Check if our downloaded version exists
    const downloadedCli = word4youCLIFilepath();
    console.log("Checking for Word4You CLI at:", downloadedCli);
    if (fs.existsSync(downloadedCli)) {
      return true;
    }

    // Otherwise check if it's in PATH
    execSync("which word4you", { stdio: "ignore" });
    return true;
  } catch (error) {
    console.warn("word4you not found", error);
    return false;
  }
}

// Get executable path for the word4you CLI
export async function getExecutablePathAsync(): Promise<string> {
  // Check if it's in PATH
  try {
    const pathOutput = execSync("which word4you", { encoding: "utf8" }).trim();

    if (pathOutput && fs.existsSync(pathOutput)) {
      return pathOutput;
    }
  } catch (error) {
    console.warn("word4you not found in PATH, will download it");
  }

  // If not found, download it
  try {
    const cliPath = await ensureCLI();
    return cliPath;
  } catch (error) {
    console.error("Failed to download word4you CLI:", error);
    throw new Error("Could not find or download word4you CLI");
  }
}

// Synchronous version for backward compatibility
export function getExecutablePath(): string {
  // Check if our downloaded version exists
  const downloadedCli = word4youCLIFilepath();
  if (fs.existsSync(downloadedCli)) {
    return downloadedCli;
  }

  // Otherwise assume it's in PATH
  return "word4you";
}

// Get preferences with proper typing
export function getPreferences(): Preferences {
  return getPreferenceValues<Preferences>();
}

// Create environment variables from preferences
export function createEnvironmentFromPreferences(): NodeJS.ProcessEnv {
  const preferences = getPreferences();
  
  return {
    ...process.env,
    // Pass Raycast preferences as environment variables for the CLI
    WORD4YOU_GEMINI_API_KEY: preferences.geminiApiKey || "",
    WORD4YOU_GEMINI_MODEL_NAME: preferences.geminiModelName || "gemini-2.0-flash-001",
    WORD4YOU_VOCABULARY_BASE_DIR: preferences.vocabularyBaseDir || "~",
    WORD4YOU_GIT_ENABLED: preferences.gitEnabled ? "true" : "false",
    WORD4YOU_GIT_REMOTE_URL: preferences.gitRemoteUrl || "",
  };
}

// Helper function to calculate SHA256 hash of a file
async function sha256FileHash(filePath: string): Promise<string> {
  return new Promise((resolve, reject) => {
    const hash = createHash("sha256");
    const stream = fs.createReadStream(filePath);

    stream.on("error", (err) => reject(err));
    stream.on("data", (chunk) => hash.update(chunk));
    stream.on("end", () => resolve(hash.digest("hex")));
  });
}

// Helper function to download a file with redirect support
async function download(url: string, destDir: string, options: { filename: string }): Promise<string> {
  const streamPipeline = promisify(pipeline);
  const destPath = path.join(destDir, options.filename);

  await mkdir(destDir, { recursive: true });

  // Function to handle HTTP requests with redirect support
  const fetchWithRedirects = (currentUrl: string, redirectCount = 0): Promise<string> => {
    return new Promise((resolve, reject) => {
      // Maximum number of redirects to follow
      const MAX_REDIRECTS = 5;

      if (redirectCount >= MAX_REDIRECTS) {
        reject(new Error(`Too many redirects (${redirectCount})`));
        return;
      }

      // Parse URL to determine if it's HTTP or HTTPS
      https
        .get(currentUrl, async (response: any) => {
          // Handle redirects (status codes 301, 302, 303, 307, 308)
          if (
            response.statusCode &&
            response.statusCode >= 300 &&
            response.statusCode < 400 &&
            response.headers.location
          ) {
            console.log(`Following redirect (${response.statusCode}) to: ${response.headers.location}`);
            // Follow the redirect
            return resolve(fetchWithRedirects(response.headers.location, redirectCount + 1));
          }

          if (response.statusCode !== 200) {
            reject(new Error(`Failed to download: ${response.statusCode}`));
            return;
          }

          try {
            await streamPipeline(response, createWriteStream(destPath));
            resolve(destPath);
          } catch (error) {
            reject(error);
          }
        })
        .on("error", reject);
    });
  };

  return fetchWithRedirects(url);
}

// Get the path to the Word4You CLI executable in a common location
function word4youCLIFilepath(): string {
  // Store the CLI in Raycast's support directory
  const dir = path.join(environment.supportPath, "cli");
  return path.join(dir, "word4you");
}

// Ensure the Word4You CLI is available, downloading it if necessary
export async function ensureCLI(): Promise<string> {
  const cli = word4youCLIFilepath();

  console.log("Ensuring Word4You CLI is available at:", cli);

  // If CLI already exists, return its path
  if (fs.existsSync(cli)) {
    return cli;
  }

  // Get platform and architecture specific information
  const platform = os.platform();
  const arch = os.arch();

  // Only support macOS and Linux
  if (platform !== "darwin" && platform !== "linux") {
    throw new Error(`Unsupported platform: ${platform}. Only macOS and Linux are supported.`);
  }

  // Map platform and architecture to GitHub release asset
  let assetName: string = "";

  if (platform === "darwin") {
    if (arch === "arm64") {
      assetName = "word4you-aarch64-apple-darwin";
    } else {
      assetName = "word4you-x86_64-apple-darwin";
    }
  } else if (platform === "linux") {
    if (arch === "x64") {
      assetName = "word4you-x86_64-unknown-linux-gnu";
    } else if (arch === "arm64") {
      assetName = "word4you-aarch64-unknown-linux-gnu";
    } else {
      throw new Error(`Unsupported architecture: ${arch} on Linux. Only x64 and arm64 are supported.`);
    }
  }

  // GitHub release URL and expected hash
  const binaryURL = `https://github.com/gnehz972/word4you/releases/download/v1.0.0/${assetName}`;

  // Expected SHA256 hashes for each asset
  // In a production environment, these hashes should be obtained from the official release page
  // and verified against a trusted source
  const expectedHashes: Record<string, string> = {
    "word4you-aarch64-apple-darwin": "5716162d81f3fce0c3f5a392cd902bfc6d44baa6c678432f9b1f2295ac97d0bf",
    "word4you-x86_64-apple-darwin": "10eea5a1dd84516eb71a930f12193ebe4ef3de16ca9281b9d8d20cfa870f7bb1",
    "word4you-x86_64-unknown-linux-gnu": "cd5b7232226e64889d7b428c5e45db46d9efe0f4ce9feeb07809a7de52de74e8",
    "word4you-aarch64-unknown-linux-gnu": "2e42a55c773cb6c14f0a6f6f01c5d536957a481d46f85c4e2b838e4a7c73bc2e",
  };

  // Create directories for download and installation
  const binDir = path.dirname(cli); // environment.supportPath/cli
  const tempDir = path.join(environment.supportPath, ".tmp");

  try {
    // Download the binary
    await download(binaryURL, tempDir, { filename: assetName });

    // Verify hash
    const downloadedFile = path.join(tempDir, assetName);
    const fileHash = await sha256FileHash(downloadedFile);

    if (fileHash !== expectedHashes[assetName]) {
      throw new Error("Hash verification failed: file hash does not match expected hash");
    }

    // Ensure the bin directory exists
    await mkdir(binDir, { recursive: true });

    // Copy the binary to the final location
    fs.copyFileSync(downloadedFile, cli);

    // Make the binary executable (chmod +x) on Unix-like systems
    await chmod(cli, "755");

    return cli;
  } catch (error) {
    console.error("Error downloading Word4You CLI:", error);
    throw new Error(`Could not download Word4You CLI: ${error}`);
  } finally {
    // Clean up temp directory
    if (fs.existsSync(tempDir)) {
      await rm(tempDir, { recursive: true, force: true });
    }
  }
}
