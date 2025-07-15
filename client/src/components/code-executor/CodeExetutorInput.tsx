import './CodeExetutorInput.css'
import CodeMirror, { EditorView } from '@uiw/react-codemirror';
import { andromeda } from '@uiw/codemirror-theme-andromeda';
import { cpp } from '@codemirror/lang-cpp';
import React, { useEffect, useState } from 'react';
import axios from 'axios';
export const CodeExetutorInput = () => {
    const [text, setText] = useState('');
    const [currentWorkspace, setCurrentWorkspace] = useState<number>(0);
    const [code, setCode] = useState('#include <stdio.h> int main() { printf("Hello World"); return 0; }');
    const [workspace, setWorkspace] = useState<String[]>(['23a629528f0e4437', '490e2609752840']);

    const executeCode = async () => {
    let response;
    let file;
        if(!workspace){
            response = await axios.post('http://127.0.1.1:5000/create_file', {
                    code: code    
                }
            )
            file = response.data.file_name.toString();
            setWorkspace([...workspace, file])
        }else{
            response = await axios.patch('http://127.0.1.1:5000/update_file', {
                file_name: workspace[currentWorkspace],
                code: code
            }
        )
        }

        if(response.request.status === 200 && workspace.length){
            const exec_response = await axios.get(`http://127.0.1.1:5000/execute_file/${workspace[currentWorkspace]}`)
            const newText = exec_response.data.code_output + exec_response.data.code_error;
            setText(newText);
        }
    }

    useEffect(()=>{
        for(let i = 0; i < workspace.length; i++) {
            const element = document.getElementById(i.toString());
            if (element) {
                element.style.boxShadow = 'inset 0 0 0 0 #1f1d238c';
                element.style.background = '#1f1d238c';
            }
        }

        const element = document.getElementById(currentWorkspace.toString());
        if (element) {
            element.style.boxShadow = 'inset 200px 0 0 0 #494553';
        }
    }, [currentWorkspace, workspace.length])


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
                    <div className='control-panel'>
                    </div>
                        <div className="list-container">
                            <ul>
                                {workspace.map((workspace, index)=> {
                                    return (
                                        <li 
                                            id={index.toString()} 
                                            onClick={()=>setCurrentWorkspace(index)
                                        }>{workspace}</li>
                                    )
                                }) }
                            </ul>
                        </div>
                </div>
            </div>
        </div>
    )
}
