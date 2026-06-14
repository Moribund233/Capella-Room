const GIPHY_API_KEY = import.meta.env.VITE_GIPHY_API_KEY || 'dc6zaTOxFJmzC'
const GIPHY_BASE = 'https://api.giphy.com/v1/gifs'

export interface GifResult {
  id: string
  title: string
  url: string
  previewUrl: string
  width: number
  height: number
}

export async function fetchTrendingGifs(): Promise<GifResult[]> {
  const res = await fetch(`${GIPHY_BASE}/trending?api_key=${GIPHY_API_KEY}&limit=24&rating=g`)
  if (!res.ok) throw new Error('Failed to fetch trending GIFs')
  const json = await res.json()
  return json.data.map(mapGifData)
}

export async function searchGifs(query: string): Promise<GifResult[]> {
  const res = await fetch(`${GIPHY_BASE}/search?api_key=${GIPHY_API_KEY}&q=${encodeURIComponent(query)}&limit=24&rating=g`)
  if (!res.ok) throw new Error('Failed to search GIFs')
  const json = await res.json()
  return json.data.map(mapGifData)
}

interface GiphyImageData {
  id: string
  title: string
  images: {
    original: { url: string; width: string; height: string }
    fixed_width_small?: { url: string }
    preview_gif?: { url: string }
  }
}

function mapGifData(data: GiphyImageData): GifResult {
  return {
    id: data.id,
    title: data.title,
    url: data.images.original.url,
    previewUrl: data.images.fixed_width_small?.url || data.images.preview_gif?.url || data.images.original.url,
    width: parseInt(data.images.original.width, 10),
    height: parseInt(data.images.original.height, 10),
  }
}
