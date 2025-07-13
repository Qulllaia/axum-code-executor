import './CodeExetutorInput.css'
import CodeMirror, { EditorView } from '@uiw/react-codemirror';
import { andromeda } from '@uiw/codemirror-theme-andromeda';
import { cpp } from '@codemirror/lang-cpp';
import React, { useState } from 'react';
import axios from 'axios';
export const CodeExetutorInput = () => {
    const [text, setText] = useState('');
    const [code, setCode] = useState('#include <stdio.h> int main() { printf("Hello World"); return 0; }');
    const [workspace, setWorkspace] = useState('');

    const executeCode = async () => {
    let response;
    let file;
    if(!workspace){
        response = await axios.post('http://127.0.1.1:5000/create_file', {
                code: code    
            }
        )
        file = response.data.file_name;
        setWorkspace(file)
    }else{
        response = await axios.patch('http://127.0.1.1:5000/update_file', {
                file_name: workspace    
            }
        )
    }

    if(response.request.status === 200 && workspace){
        const exec_response = await axios.get(`http://127.0.1.1:5000/execute_file/${workspace}`)
        const newText = exec_response.data.code_output + exec_response.data.code_error;
        setText(newText);

        console.log(exec_response)
    }
    console.log(response);
}


    return (
        <div className='container'>
            <div className='code-container'>
                <div className='control-panel'>
                    <button className='execute-button'
                    onClick={executeCode}
                    > Execute </button>
                </div>
                 <CodeMirror
                    className='code-area'
                    theme={andromeda}
                    extensions={[cpp(),
                        EditorView.theme({
                            ".cm-scroller": { overflow: "auto" },
                        })
                    ]}
                    basicSetup={{
                        highlightActiveLine: true,
                    }}    
                    value={code}
                    onChange={(e)=> setCode(e)}
                    />
                <textarea className='output-container'
                    value={text}
                >
                </textarea>
            </div>
            <div className='work-spaces-container'>
                <div className='work-spaces'>

                </div>
            </div>
        </div>
    )
}
