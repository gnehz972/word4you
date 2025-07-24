import fs from "fs";
import { getCliFilepath, getDownloadUrl } from "../config";
import { downloadFile, verifyFileHash } from "../utils/downloadUtils";
import { environment } from "@raycast/api";
import path from "path";
import { chmod, mkdir, rm } from "fs/promises";

// Check if word4you CLI is installed
export function isCliInstalled(): boolean {
    try {
        // Check if our downloaded version exists
        const downloadedCli = getCliFilepath();
        console.log("Checking for Word4You CLI at:", downloadedCli);
        if (fs.existsSync(downloadedCli)) {
            return true;
        }

        // Otherwise check if it's in PATH
        const { execSync } = require("child_process");
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
        const { execSync } = require("child_process");
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

// Ensure the Word4You CLI is available, downloading it if necessary
export async function ensureCLI(): Promise<string> {
    const cli = getCliFilepath();

    console.log("Ensuring Word4You CLI is available at:", cli);

    // If CLI already exists, return its path
    if (fs.existsSync(cli)) {
        return cli;
    }

    // Get download configuration from config
    const { url: binaryURL, assetName, expectedHash } = getDownloadUrl();

    // Create directories for download and installation
    const binDir = path.dirname(cli);
    const tempDir = path.join(environment.supportPath, ".tmp");

    try {
        // Download the binary
        const downloadedFile = await downloadFile(binaryURL, tempDir, { filename: assetName });

        // Verify hash
        const fileHash = await verifyFileHash(downloadedFile);

        if (fileHash !== expectedHash) {
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