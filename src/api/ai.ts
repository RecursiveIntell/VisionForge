import { invoke } from "@tauri-apps/api/core";

export async function tagImage(imageId: string): Promise<string[]> {
  return invoke("tag_image", { imageId });
}

export async function captionImage(imageId: string): Promise<string> {
  return invoke("caption_image", { imageId });
}
