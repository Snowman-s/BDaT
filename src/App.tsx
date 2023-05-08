import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import { listen } from "@tauri-apps/api/event";
import "./App.css";

function binariesToJSX(binaries: number[]): JSX.Element {
  let copyData: String[][] = [];

  for (let i = 0; i < binaries.length; i++) {
    if (copyData.length == 0 || copyData[copyData.length - 1].length == 8) {
      copyData.push([]);
    }
    const bToStr = binaries[i].toString(16);

    copyData[copyData.length - 1].push(
      bToStr.length == 1 ? "0" + bToStr : bToStr
    );
  }

  return (
    <table>
      <tbody>
        {copyData.map((ary, i) => (
          <tr key={"tr" + i}>
            {ary.map((dat, i2) => (
              <td key={"td" + i2}>{dat}</td>
            ))}
          </tr>
        ))}
      </tbody>
    </table>
  );
}

function App() {
  const [binaries, setBinaries] = useState<number[]>([]);
  const [filePath, setFilePath] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    const unlisten = listen("open", async (_event) => {
      (invoke("find_binary_file_path") as Promise<string | null>).then((s) =>
        setFilePath(s)
      );
    });

    return () => {
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
      setLoading(false);
      if (!ignore) {
        if (Array.isArray(binaryOrError)) {
          setBinaries(binaryOrError);
        } else {
          console.error(binaryOrError);
        }
      }
    });

    return () => {
      ignore = true;
    };
  }, [filePath]);

  return (
    <div className="container">
      {loading ? <p>ロード中...</p> : binariesToJSX(binaries)}
    </div>
  );
}

export default App;
