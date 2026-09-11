import * as fs from "node:fs";
import * as path from "node:path";

export interface FileMetadata {
  id: string;
  filename: string;
  size: number;
  sha256: string;
  content_type: string;
  created_at: string;
}

export interface FileListResponse {
  success: boolean;
  count: number;
  files: FileMetadata[];
}

export interface UploadResponse {
  success: boolean;
  file: FileMetadata;
}

export interface DeleteResponse {
  success: boolean;
  message: string;
}

export interface HealthResponse {
  status: string;
  files_count: number;
  total_bytes: number;
}

export class FileStorageClient {
  private baseUrl: string;

  constructor(baseUrl: string = "http://127.0.0.1:3000") {
    this.baseUrl = baseUrl.replace(/\/+$/, "");
  }

  async health(): Promise<HealthResponse> {
    const res = await fetch(`${this.baseUrl}/health`);
    if (!res.ok) {
      throw new Error(`Health check failed: ${res.statusText}`);
    }
    return res.json() as Promise<HealthResponse>;
  }

  async uploadFile(filePath: string): Promise<FileMetadata> {
    const resolvedPath = path.resolve(filePath);
    if (!fs.existsSync(resolvedPath)) {
      throw new Error(`File not found: ${resolvedPath}`);
    }

    const filename = path.basename(resolvedPath);
    const fileBuffer = await fs.promises.readFile(resolvedPath);
    const blob = new Blob([fileBuffer]);

    const formData = new FormData();
    formData.append("file", blob, filename);

    const res = await fetch(`${this.baseUrl}/files/upload`, {
      method: "POST",
      body: formData,
    });

    if (!res.ok) {
      const errText = await res.text();
      throw new Error(`Upload failed (${res.status}): ${errText}`);
    }

    const data = (await res.json()) as UploadResponse;
    return data.file;
  }

  async listFiles(): Promise<FileMetadata[]> {
    const res = await fetch(`${this.baseUrl}/files`);
    if (!res.ok) {
      throw new Error(`Failed to list files: ${res.statusText}`);
    }
    const data = (await res.json()) as FileListResponse;
    return data.files;
  }

  async getFileInfo(id: string): Promise<FileMetadata> {
    const res = await fetch(`${this.baseUrl}/files/${id}/info`);
    if (!res.ok) {
      throw new Error(`Failed to get file info: ${res.statusText}`);
    }
    return res.json() as Promise<FileMetadata>;
  }

  async downloadFile(id: string, outputFilePath: string): Promise<string> {
    const res = await fetch(`${this.baseUrl}/files/${id}`);
    if (!res.ok) {
      throw new Error(`Download failed: ${res.statusText}`);
    }

    const arrayBuffer = await res.arrayBuffer();
    const buffer = Buffer.from(arrayBuffer);
    const targetPath = path.resolve(outputFilePath);

    const parentDir = path.dirname(targetPath);
    if (!fs.existsSync(parentDir)) {
      fs.mkdirSync(parentDir, { recursive: true });
    }

    await fs.promises.writeFile(targetPath, buffer);
    return targetPath;
  }

  async deleteFile(id: string): Promise<string> {
    const res = await fetch(`${this.baseUrl}/files/${id}`, {
      method: "DELETE",
    });
    if (!res.ok) {
      throw new Error(`Delete failed: ${res.statusText}`);
    }
    const data = (await res.json()) as DeleteResponse;
    return data.message;
  }
}
