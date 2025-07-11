import React, { useState, useCallback, useEffect } from "react"

interface ResizerProps {
  onResize: (leftWidth: number) => void
  initialLeftWidth?: number
}

const Resizer: React.FC<ResizerProps> = ({ onResize, initialLeftWidth = 50 }) => {
  const [isResizing, setIsResizing] = useState(false)

  const handleMouseDown = useCallback(() => {
    setIsResizing(true)
  }, [])

  const handleMouseMove = useCallback(
    (e: MouseEvent) => {
      if (!isResizing) return
      
      const containerWidth = window.innerWidth
      const leftWidth = (e.clientX / containerWidth) * 100
      
      // 最小20%、最大80%に制限
      const clampedWidth = Math.min(Math.max(leftWidth, 20), 80)
      onResize(clampedWidth)
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
      document.body.style.cursor = "col-resize"
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
      className="resizer"
      onMouseDown={handleMouseDown}
      style={{
        width: "4px",
        background: isResizing ? "#80cbc4" : "#2a2a2a",
        cursor: "col-resize",
        userSelect: "none",
        transition: isResizing ? "none" : "background 0.2s ease",
      }}
    />
  )
}

export default Resizer