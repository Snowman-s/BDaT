export type ParsedData = {
  explain: string;
  minIndex: number;
  maxIndex: number;
  children: {data: ParsedData, name: string }[]
};
