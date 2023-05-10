import React, { useRef, useState } from "react";

export const Divider = (props: {
  onMouseDown: React.MouseEventHandler<HTMLDivElement>;
}) => {
  const { onMouseDown: onMouseHoldDown } = props;

  return (
    <div
      {...props}
      onMouseDown={onMouseHoldDown}
      style={{
        border: "5px solid black",
        cursor: "row-resize",
      }}
    />
  );
};

export const SplitPane = (props: {
  child1: JSX.Element;
  child2: JSX.Element;
}) => {
  const { child1, child2 } = props;
  const topRef = useRef<HTMLDivElement | null>(null);

  // (percent)
  const [leftWidth, setLeftWidth] = useState<number>(50);

  const dividerY = useRef<null | number>(null);

  return (
    <div
      onMouseUp={(_event) => {
        dividerY.current = null;
      }}
      onMouseMove={(event) => {
        if (dividerY.current == null || topRef.current == null) return;

        const containerRect = topRef.current.getBoundingClientRect();
        const y = event.clientY - containerRect.top;
        setLeftWidth((y / containerRect.height) * 100);
      }}
      ref={topRef}
      style={{
        width: "100vw",
        height: "90vh",
        display: "flex",
        flexDirection: "column",
      }}
    >
      <div style={{ height: `${leftWidth}%`, overflow: "auto" }}>{child1}</div>
      <Divider
        onMouseDown={(event) => {
          dividerY.current = event.clientY;
        }}
      />
      <div style={{ height: `${100 - leftWidth}%`, overflow: "auto" }}>
        {child2}
      </div>
    </div>
  );
};
