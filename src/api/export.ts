import { invoke } from "@tauri-apps/api/core";
import type { GalleryFilter } from "../types";

export async function exportImages(
  imageIds: string[],
  outputPath: string,
): Promise<void> {
  return invoke("export_images", { imageIds, outputPath });
}

export async function exportGallery(
  filter: GalleryFilter,
  outputPath: string,
): Promise<number> {
  return invoke("export_gallery", { filter, outputPath });
}
