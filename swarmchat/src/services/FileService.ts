/**
 * FileService — handles file upload, download, CID computation.
 *
 * In production, integrates with IPFS/Bitswap for decentralized file transfer.
 * For now, uses relay as file proxy with local file system operations.
 */

import { FileTransfer } from '../types';
import { IdentityService } from './IdentityService';
import { v4 as uuidv4 } from 'uuid';
import RNFS from 'react-native-fs';

export const FileService = {
  /**
   * Pick and prepare a file for sending.
   */
  async prepareFile(
    localPath: string,
    fileName: string,
    mime: string,
    size: number,
  ): Promise<{ cid: string; thumbnailCid?: string }> {
    // In production: compute IPFS CID from file content
    // For now: generate a placeholder CID
    const cid = `bafybei${uuidv4().replace(/-/g, '')}`;

    // For images, generate thumbnail
    let thumbnailCid: string | undefined;
    if (mime.startsWith('image/')) {
      thumbnailCid = `bafybei_thumb_${uuidv4().replace(/-/g, '').substring(0, 40)}`;
    }

    return { cid, thumbnailCid };
  },

  /**
   * Start uploading a file to the network.
   */
  async uploadFile(
    localPath: string,
    fileName: string,
    mime: string,
    peerDid: string,
  ): Promise<FileTransfer> {
    const stat = await RNFS.stat(localPath);

    const transfer: FileTransfer = {
      id: uuidv4(),
      cid: '',
      fileName,
      mime,
      size: Number(stat.size),
      peerDid,
      direction: 'upload',
      progress: 0,
      status: 'pending',
      localPath,
    };

    // Compute CID
    const { cid } = await this.prepareFile(localPath, fileName, mime, Number(stat.size));
    transfer.cid = cid;

    // In production: upload to IPFS and announce CID on DHT
    // For now: simulate progress
    transfer.status = 'transferring';
    this.simulateProgress(transfer);

    return transfer;
  },

  /**
   * Download a file from CID.
   */
  async downloadFile(
    cid: string,
    fileName: string,
    mime: string,
    size: number,
    peerDid: string,
  ): Promise<FileTransfer> {
    const downloadDir = `${RNFS.DocumentDirectoryPath}/downloads`;
    await RNFS.mkdir(downloadDir);

    const transfer: FileTransfer = {
      id: uuidv4(),
      cid,
      fileName,
      mime,
      size,
      peerDid,
      direction: 'download',
      progress: 0,
      status: 'pending',
      localPath: `${downloadDir}/${fileName}`,
    };

    // In production: fetch from IPFS/Bitswap
    transfer.status = 'transferring';
    this.simulateProgress(transfer);

    return transfer;
  },

  /**
   * Share a file from another app (Android intent / iOS share sheet).
   */
  async handleSharedFile(uri: string, mime: string, fileName: string): Promise<{
    localPath: string;
    fileName: string;
    mime: string;
    size: number;
  }> {
    const destPath = `${RNFS.TemporaryDirectoryPath}/${fileName}`;
    await RNFS.copyFile(uri, destPath);
    const stat = await RNFS.stat(destPath);

    return {
      localPath: destPath,
      fileName,
      mime,
      size: Number(stat.size),
    };
  },

  /**
   * Get the local path for a downloaded file by CID.
   */
  getLocalPath(cid: string, fileName: string): string {
    return `${RNFS.DocumentDirectoryPath}/downloads/${fileName}`;
  },

  // ---- Internal ----

  simulateProgress(transfer: FileTransfer): void {
    // In production, this would track actual IPFS transfer progress
    let progress = 0;
    const interval = setInterval(() => {
      progress += Math.random() * 20;
      if (progress >= 100) {
        progress = 100;
        transfer.status = 'completed';
        transfer.progress = 100;
        clearInterval(interval);
      }
      transfer.progress = Math.round(progress);
    }, 500);
  },
};

export default FileService;
