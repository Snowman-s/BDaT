import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import { listen } from "@tauri-apps/api/event";
import "./App.css";
import { SplitPane } from "./SplitPane";
import { ParsedData } from "./ParsedData";
import { parse } from "path";

function binaryWindow(
  binaries: number[],
  selected: SelectedFragment | null,
  onBinaryClicked: (index: number) => void
) {
  let tableData: { data: String; index: number }[][] = [];

  for (let i = 0; i < binaries.length; i++) {
    if (tableData.length == 0 || tableData[tableData.length - 1].length == 16) {
      tableData.push([]);
    }
    const bToStr = binaries[i].toString(16);

    tableData[tableData.length - 1].push({
      data: bToStr.length == 1 ? "0" + bToStr : bToStr,
      index: i,
    });
  }

  return (
    <table>
      <tbody>
        {tableData.map((ary, i) => (
          <tr key={"tr" + i}>
            {ary.map((dat, i2) => (
              <td
                key={"td" + i2}
                onMouseDown={(_) => onBinaryClicked(dat.index)}
                style={
                  selected != null &&
                    selected.binary.minIndex <= dat.index &&
                    dat.index < selected.binary.maxIndex
                    ? { background: "blue" }
                    : {}
                }
              >
                {dat.data}
              </td>
            ))}
          </tr>
        ))}
      </tbody>
    </table>
  );
}

function analysisWindow(
  parsed: ParsedData | null,
  selected: SelectedFragment | null,
  onAnalysisClicked: (p: ParsedData) => void
) {
  function accordion(name: string, parsed: ParsedData) {
    if (parsed.children.length === 0) {
      return (
        <div
          style={
            selected != null && selected.parsed == parsed
              ? { background: "blue" }
              : {}
          }
          onMouseDown={(_) => onAnalysisClicked(parsed)}
        >
          {(name == "" ? "" : name + "： ") + parsed.explain}
        </div>
      );
    } else {
      return (
        <details>
          <summary>{(name == "" ? "" : name + "： ") + parsed.explain}</summary>
          {parsed.children.map((c) => accordion(c.name, c.data))}
        </details>
      );
    }
  }
  return parsed == null ? <></> : accordion("", parsed);
}

function binariesToJSX(
  binaries: number[],
  parsed: ParsedData | null,
  selected: SelectedFragment | null,
  onBinaryClicked: (index: number) => void,
  onAnalysisClicked: (p: ParsedData) => void
): JSX.Element {
  return (
    <SplitPane
      child1={
        <div
          style={{
            backgroundColor: "whitesmoke",
            color: "black",
            minHeight: "100%",
          }}
        >
          {binaryWindow(binaries, selected, onBinaryClicked)}
        </div>
      }
      child2={
        <div
          style={{
            backgroundColor: "whitesmoke",
            color: "black",
            minHeight: "100%",
          }}
        >
          {analysisWindow(parsed, selected, onAnalysisClicked)}
        </div>
      }
    />
  );
}

type SelectedFragment = {
  binary: { minIndex: number; maxIndex: number };
  parsed: ParsedData;
};

function createSelectedFragment(
  parsed: ParsedData | null,
  binaryIndexOrAnalysis: number | ParsedData
): SelectedFragment | null {
  if (parsed == null) return null;


  if (typeof binaryIndexOrAnalysis == "number") {
    const binaryIndex = binaryIndexOrAnalysis;
    let nowLooking = parsed;
    while (nowLooking.children.length !== 0) {
      let nowLookingOrUndefined = nowLooking.children.find(
        (child) =>
          child.data.minIndex <= binaryIndex &&
          binaryIndex <= child.data.maxIndex
      );
      if (nowLookingOrUndefined === undefined) return null;
      nowLooking = nowLookingOrUndefined.data;
    }

    return {
      binary: { minIndex: nowLooking.minIndex, maxIndex: nowLooking.maxIndex },
      parsed: nowLooking,
    };
  } else {
    return {
      binary: {
        minIndex: binaryIndexOrAnalysis.minIndex,
        maxIndex: binaryIndexOrAnalysis.maxIndex,
      },
      parsed: binaryIndexOrAnalysis,
    };
  }
}

function App() {
  const [binaries, setBinaries] = useState<number[]>([]);
  const [parsed, setParsed] = useState<ParsedData | null>(null);
  const [filePath, setFilePath] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const [parserList, setParserList] = useState<string[]>([]);
  const [parserValue, setParserValue] = useState<string | null>(null);

  const [selectedFragment, setSelectedFragment] =
    useState<SelectedFragment | null>(null);

  useEffect(() => {
    let ignore = false;

    const unlisten = listen("open", async (_event) => {
      (invoke("find_binary_file_path") as Promise<string | null>).then((s) =>
        setFilePath(s)
      );
    });

    (invoke("get_parser_list") as Promise<string[]>).then((p) => {
      if (!ignore) {
        setParserList(p);
        setParserValue(p[0]);
      }
    });

    return () => {
      ignore = true;
      unlisten.then((f) => f());
    };
  }, []);

  useEffect(() => {
    if (filePath == null) {
      return;
    }

    let ignore = false;
    setLoading(true);
    (
      invoke("load_binary_file", { path: filePath }) as Promise<
        number[] | String
      >
    ).then((binaryOrError) => {
      if (!ignore) {
        if (Array.isArray(binaryOrError)) {
          setBinaries(binaryOrError);
        } else {
          console.error(binaryOrError);
        }
        setLoading(false);
      } else {
        setLoading(false);
      }
    });

    return () => {
      ignore = true;
    };
  }, [filePath]);

  useEffect(() => {
    if (binaries == null || parserValue == null) {
      setParsed(null);
      return;
    }

    let ignore = false;
    setLoading(true);
    (
      invoke("parse", {
        parser: parserValue,
        data: binaries,
      }) as Promise<ParsedData | string>
    ).then((parsed) => {
      if (!ignore && typeof parsed !== "string") {
        console.log(parsed);
        setParsed(parsed);
      }
      setLoading(false);
    });

    return () => {
      ignore = true;
    };
  }, [binaries, parserValue]);

  return (
    <div className="container">
      {loading ? (
        <p>ロード中...</p>
      ) : (
        binariesToJSX(
          binaries,
          parsed,
          selectedFragment,
          (i) => {
            setSelectedFragment(createSelectedFragment(parsed, i));
          },
          (p) => {
            setSelectedFragment(createSelectedFragment(parsed, p));
          }
        )
      )}
      <select
        onChange={(e) => setParserValue(e.target.value as unknown as string)}
        defaultValue={parserValue!}
      >
        {parserList.map((parser, i) => (
          <option key={i} value={parser}>{parser}</option>
        ))}
      </select>
    </div>
  );
}

export default App;
