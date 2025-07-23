import type React from "react"
import { useCallback, useEffect, useState } from "react"

interface VerticalResizerProps {
  onResize: (topHeight: number) => void
}

const VerticalResizer: React.FC<VerticalResizerProps> = ({ onResize }) => {
  const [isResizing, setIsResizing] = useState(false)

  const handleMouseDown = useCallback(() => {
    setIsResizing(true)
  }, [])

  const handleTouchStart = useCallback(() => {
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

  const handleTouchMove = useCallback(
    (e: TouchEvent) => {
      if (!isResizing) return
      e.preventDefault()

      const touch = e.touches[0]
      const containerHeight = window.innerHeight - 100 // ヘッダー分を除く
      const topHeight = ((touch.clientY - 100) / containerHeight) * 100

      // 最小30%、最大80%に制限
      const clampedHeight = Math.min(Math.max(topHeight, 30), 80)
      onResize(clampedHeight)
    },
    [isResizing, onResize]
  )

  const handleMouseUp = useCallback(() => {
    setIsResizing(false)
  }, [])

  const handleTouchEnd = useCallback(() => {
    setIsResizing(false)
  }, [])

  useEffect(() => {
    if (isResizing) {
      document.addEventListener("mousemove", handleMouseMove)
      document.addEventListener("mouseup", handleMouseUp)
      document.addEventListener("touchmove", handleTouchMove, {
        passive: false,
      })
      document.addEventListener("touchend", handleTouchEnd)
      document.body.style.cursor = "row-resize"
      document.body.style.userSelect = "none"
    } else {
      document.removeEventListener("mousemove", handleMouseMove)
      document.removeEventListener("mouseup", handleMouseUp)
      document.removeEventListener("touchmove", handleTouchMove)
      document.removeEventListener("touchend", handleTouchEnd)
      document.body.style.cursor = ""
      document.body.style.userSelect = ""
    }

    return () => {
      document.removeEventListener("mousemove", handleMouseMove)
      document.removeEventListener("mouseup", handleMouseUp)
      document.removeEventListener("touchmove", handleTouchMove)
      document.removeEventListener("touchend", handleTouchEnd)
      document.body.style.cursor = ""
      document.body.style.userSelect = ""
    }
  }, [
    isResizing,
    handleMouseMove,
    handleMouseUp,
    handleTouchMove,
    handleTouchEnd,
  ])

  return (
    <div
      className="vertical-resizer"
      onMouseDown={handleMouseDown}
      onTouchStart={handleTouchStart}
      style={{
        height: "12px",
        background: isResizing ? "#80cbc4" : "#2a2a2a",
        cursor: "row-resize",
        userSelect: "none",
        transition: isResizing ? "none" : "background 0.2s ease",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        position: "relative",
      }}
    >
      {/* 視覚的なハンドルバー */}
      <div
        style={{
          width: "40px",
          height: "3px",
          backgroundColor: isResizing ? "#ffffff" : "#666",
          borderRadius: "2px",
          position: "relative",
        }}
      >
        <div
          style={{
            position: "absolute",
            top: "-4px",
            left: "0",
            width: "100%",
            height: "3px",
            backgroundColor: isResizing ? "#ffffff" : "#666",
            borderRadius: "2px",
          }}
        />
        <div
          style={{
            position: "absolute",
            top: "4px",
            left: "0",
            width: "100%",
            height: "3px",
            backgroundColor: isResizing ? "#ffffff" : "#666",
            borderRadius: "2px",
          }}
        />
      </div>
    </div>
  )
}

export default VerticalResizer
