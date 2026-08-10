export interface CollectionNode {
  id: string
  name: string
  count: number
  children?: CollectionNode[]
}
