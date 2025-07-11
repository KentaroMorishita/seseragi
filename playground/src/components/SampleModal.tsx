import React, { useState } from "react"
import {
  allCodeSections,
  sectionsByCategory,
  type CodeSection,
} from "../samples/examples-data"

interface SampleModalProps {
  isOpen: boolean
  onClose: () => void
  onSelectSample: (code: string) => void
}

const SampleModal: React.FC<SampleModalProps> = ({
  isOpen,
  onClose,
  onSelectSample,
}) => {
  const [selectedCategory, setSelectedCategory] = useState<
    "basics" | "intermediate" | "advanced"
  >("basics")
  const [openFiles, setOpenFiles] = useState<Set<string>>(new Set())
  const [previewSection, setPreviewSection] = useState<CodeSection | null>(null)

  if (!isOpen) return null

  const categories = [
    { key: "basics", label: "基礎編" },
    { key: "intermediate", label: "中級編" },
    { key: "advanced", label: "上級編" },
  ] as const

  // 選択されたカテゴリのセクションを取得
  const sectionsInCategory = sectionsByCategory[selectedCategory]

  // ファイル別にグループ化
  const fileGroups = sectionsInCategory.reduce(
    (acc, section) => {
      const fileKey = section.fileTitle
      if (!acc[fileKey]) acc[fileKey] = []
      acc[fileKey].push(section)
      return acc
    },
    {} as Record<string, CodeSection[]>
  )

  const handleSectionSelect = (section: CodeSection) => {
    onSelectSample(section.code)
    onClose()
  }

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h2>Seseragi Examples</h2>
          <button className="close-button" onClick={onClose}>
            ×
          </button>
        </div>

        <div className="modal-body">
          {/* カテゴリ選択 */}
          <div className="category-tabs">
            {categories.map((cat) => (
              <button
                key={cat.key}
                className={`category-tab ${selectedCategory === cat.key ? "active" : ""}`}
                onClick={() => {
                  setSelectedCategory(cat.key)
                  setOpenFiles(new Set())
                  setPreviewSection(null)
                }}
              >
                {cat.label}
              </button>
            ))}
          </div>

          <div className="content-area">
            {/* ファイル・セクション一覧 */}
            <div className="file-list">
              {Object.entries(fileGroups).map(([fileName, sections]) => (
                <div key={fileName} className="file-group">
                  <div
                    className={`file-header ${openFiles.has(fileName) ? "active" : ""}`}
                    onClick={() => {
                      const newOpenFiles = new Set(openFiles)
                      if (openFiles.has(fileName)) {
                        newOpenFiles.delete(fileName)
                      } else {
                        newOpenFiles.add(fileName)
                      }
                      setOpenFiles(newOpenFiles)
                    }}
                  >
                    {fileName}
                    <span className="section-count">
                      ({sections.length}セクション)
                    </span>
                  </div>

                  {openFiles.has(fileName) && (
                    <div className="section-list">
                      {sections.map((section) => (
                        <div
                          key={section.id}
                          className={`section-item ${previewSection?.id === section.id ? "previewing" : ""}`}
                          onMouseEnter={() => setPreviewSection(section)}
                          onClick={() => handleSectionSelect(section)}
                        >
                          <div className="section-title">{section.title}</div>
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              ))}
            </div>

            {/* プレビューエリア */}
            {previewSection && (
              <div className="preview-area">
                <div className="preview-header">
                  <h4>{previewSection.title}</h4>
                  <button
                    className="use-button"
                    onClick={() => handleSectionSelect(previewSection)}
                  >
                    このコードを使用
                  </button>
                </div>
                <pre className="preview-code">{previewSection.code}</pre>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  )
}

export default SampleModal
