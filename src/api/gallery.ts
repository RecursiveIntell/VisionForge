import { invoke } from "@tauri-apps/api/core";
import type { ImageEntry, GalleryFilter } from "../types";

export async function getGalleryImages(
  filter: GalleryFilter,
): Promise<ImageEntry[]> {
  return invoke("get_gallery_images", { filter });
}

export async function getImage(id: string): Promise<ImageEntry | null> {
  return invoke("get_image", { id });
}

export async function deleteImage(id: string): Promise<void> {
  return invoke("delete_image", { id });
}

export async function restoreImage(id: string): Promise<void> {
  return invoke("restore_image", { id });
}

export async function permanentlyDeleteImage(id: string): Promise<void> {
  return invoke("permanently_delete_image", { id });
}

export async function updateImageRating(
  id: string,
  rating: number | null,
): Promise<void> {
  return invoke("update_image_rating", { id, rating });
}

export async function updateImageFavorite(
  id: string,
  favorite: boolean,
): Promise<void> {
  return invoke("update_image_favorite", { id, favorite });
}

export async function updateCaption(
  id: string,
  caption: string,
): Promise<void> {
  return invoke("update_caption", { id, caption });
}

export async function updateImageNote(
  id: string,
  note: string,
): Promise<void> {
  return invoke("update_image_note", { id, note });
}

export async function addTag(
  imageId: string,
  tag: string,
  source: string,
): Promise<void> {
  return invoke("add_tag", { imageId, tag, source });
}

export async function removeTag(
  imageId: string,
  tagId: number,
): Promise<void> {
  return invoke("remove_tag", { imageId, tagId });
}

export async function getImageLineage(
  imageId: string,
): Promise<string | null> {
  return invoke("get_image_lineage", { imageId });
}

export async function getImageFilePath(filename: string): Promise<string> {
  return invoke("get_image_file_path", { filename });
}

export async function getThumbnailFilePath(filename: string): Promise<string> {
  return invoke("get_thumbnail_file_path", { filename });
}
