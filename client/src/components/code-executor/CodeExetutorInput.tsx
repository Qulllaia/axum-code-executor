import './CodeExetutorInput.css'
import CodeMirror, { EditorView } from '@uiw/react-codemirror';
import { andromeda } from '@uiw/codemirror-theme-andromeda';
import { cpp } from '@codemirror/lang-cpp';
import React, { useEffect, useState } from 'react';
import axios from 'axios';
import { Workspace } from '../../types/types';
export const CodeExetutorInput = () => {
    const [text, setText] = useState('');
    const [currentWorkspace, setCurrentWorkspace] = useState<number>(0);
    const [code, setCode] = useState('#include <stdio.h> int main() { printf("Hello World"); return 0; }');
    const [workspace, setWorkspace] = useState<String[]>(['23a629528f0e4437', '490e2609752840']);
    const [userId, setUserId] = useState<number>(1);
    const [workspaceList, setWorkspaceList] = useState< Workspace[]>([]);

    const currentCode = workspaceList[currentWorkspace]?.code || '';

    const executeCode = async () => {
        let response;
        let file;
        if(!workspace){
            response = await axios.post('http://127.0.1.1:5000/create_file', {
                    code: currentCode    
                }
            )
            file = response.data.file_name.toString();
            setWorkspace([...workspace, file])
        }else{
            response = await axios.patch('http://127.0.1.1:5000/update_file', {
                file_name: workspaceList[currentWorkspace].workspace_uid,
                code: currentCode
            }
        )
        }

        if(response.request.status === 200 && workspace.length){
            console.log(currentWorkspace);
            const exec_response = await axios.get(`http://127.0.1.1:5000/execute_file/${workspaceList[currentWorkspace].workspace_uid}`)
            const newText = exec_response.data.code_output + exec_response.data.code_error;
            setText(newText);
        }
    }

    const fetchData = async () => {
        const response = await axios.get(`http://127.0.1.1:5000/get_files/${userId}`)
        const wl = response.data?.row.map((item: Workspace) => {
            return {
                code: item.code, 
                user_id: item.user_id, 
                workspace_name: item.workspace_name,
                workspace_uid: item.workspace_uid,
            } as Workspace
        }) 
        console.log(wl);
        setWorkspaceList(wl);
        console.log(workspaceList);

    } 

    useEffect(()=>{
        if(!workspaceList.length)
            fetchData();

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
                    value={currentCode as string}  
                    onChange={(value) => {
                        const updatedList = [...workspaceList];
                        if (updatedList[currentWorkspace]) {
                            updatedList[currentWorkspace].code = value;
                            setWorkspaceList(updatedList);
                        }
                    }}
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
                                {workspaceList.map((workspace, index)=> {
                                    return (
                                        <li 
                                            id={index.toString()} 
                                            onClick={()=>setCurrentWorkspace(index)
                                        }>{workspace.workspace_name}</li>
                                    )
                                }) }
                            </ul>
                        </div>
                </div>
            </div>
        </div>
    )
}
