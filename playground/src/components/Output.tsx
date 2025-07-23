import type React from "react"

interface OutputProps {
  output: string
  error: string | null
}

const Output: React.FC<OutputProps> = ({ output, error }) => {
  return (
    <div className="output-content">
      {error ? (
        <pre className="error">{error}</pre>
      ) : output ? (
        <pre className="success">{output}</pre>
      ) : (
        <div style={{ color: "#666" }}>Click "Run" to execute your code</div>
      )}
    </div>
  )
}

export default Output
