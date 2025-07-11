import React, { useState, useCallback, useEffect } from "react"

interface VerticalResizerProps {
  onResize: (topHeight: number) => void
  initialTopHeight?: number
}

const VerticalResizer: React.FC<VerticalResizerProps> = ({ onResize, initialTopHeight = 60 }) => {
  const [isResizing, setIsResizing] = useState(false)

  const handleMouseDown = useCallback(() => {
    setIsResizing(true)
  }, [])

  const handleMouseMove = useCallback(
    (e: MouseEvent) => {
      if (!isResizing) return
      
      const containerHeight = window.innerHeight - 100 // ヘッダー分を除く
      const topHeight = ((e.clientY - 100) / containerHeight) * 100
      
      // 最小30%、最大80%に制限
      const clampedHeight = Math.min(Math.max(topHeight, 30), 80)
      onResize(clampedHeight)
    },
    [isResizing, onResize]
  )

  const handleMouseUp = useCallback(() => {
    setIsResizing(false)
  }, [])

  useEffect(() => {
    if (isResizing) {
      document.addEventListener("mousemove", handleMouseMove)
      document.addEventListener("mouseup", handleMouseUp)
      document.body.style.cursor = "row-resize"
      document.body.style.userSelect = "none"
    } else {
      document.removeEventListener("mousemove", handleMouseMove)
      document.removeEventListener("mouseup", handleMouseUp)
      document.body.style.cursor = ""
      document.body.style.userSelect = ""
    }

    return () => {
      document.removeEventListener("mousemove", handleMouseMove)
      document.removeEventListener("mouseup", handleMouseUp)
      document.body.style.cursor = ""
      document.body.style.userSelect = ""
    }
  }, [isResizing, handleMouseMove, handleMouseUp])

  return (
    <div
      className="vertical-resizer"
      onMouseDown={handleMouseDown}
      style={{
        height: "4px",
        background: isResizing ? "#80cbc4" : "#2a2a2a",
        cursor: "row-resize",
        userSelect: "none",
        transition: isResizing ? "none" : "background 0.2s ease",
      }}
    />
  )
}

export default VerticalResizer