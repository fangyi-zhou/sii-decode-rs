import { useEffect, useRef, useState } from 'react'
import './App.css'
import { decode } from 'sii-decode-rs'

function App() {
  const [file, setFile] = useState<File | null>(null)
  const textAreaRef = useRef<HTMLTextAreaElement>(null);
  const downloadRef = useRef<HTMLAnchorElement>(null);

  const handleFile = (event: React.ChangeEvent<HTMLInputElement>) => {
    if (event.target.files && event.target.files.length > 0) {
      const selectedFile = event.target.files[0]
      setFile(selectedFile)
    }
  }

  useEffect(() => {
    if (file) {
      const reader = new FileReader()
      reader.onload = (e) => {
        const arrayBuffer = e.target?.result as ArrayBuffer
        const bytes = new Uint8Array(arrayBuffer)
        const decoded = decode(bytes)
        if (textAreaRef.current) {
          textAreaRef.current.value = decoded
        }
        if (downloadRef.current) {
          const blob = new Blob([decoded], { type: 'text/plain' })
          const url = URL.createObjectURL(blob)
          downloadRef.current.href = url
          downloadRef.current.download = file.name.replace('.sii', '-decoded.sii')
          downloadRef.current.textContent = 'Download'
        }
      }
      reader.readAsArrayBuffer(file)
    }
  }, [file])

  return (
    <>
      <h1>SII Decode (beta)</h1>
      <p>Select your file</p>
      <div>
        <input type="file" id="file" onChange={handleFile} />
      </div>
      <br />
      <textarea id="output" rows={20} cols={50} ref={textAreaRef} />
      <div>
        <a href="#" ref={downloadRef}>Download</a>
      </div>
      <p className="read-the-docs">
        Your file will not be uploaded to the server. It will be decoded in your browser.
      </p>
    </>
  )
}

export default App
