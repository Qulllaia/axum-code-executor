import './CodeExetutorInput.css'
import CodeMirror, { EditorView } from '@uiw/react-codemirror';
import { andromeda } from '@uiw/codemirror-theme-andromeda';
import { cpp } from '@codemirror/lang-cpp';
import React, { useEffect, useState } from 'react';
import axios from 'axios';
import { Workspace } from '../../types/types';
export const CodeExetutorInput = () => {
    const [text, setText] = useState('');
    const [isLoading, setIsLoading] = useState<boolean>(false);
    const [currentWorkspace, setCurrentWorkspace] = useState<number>(0);
    const [code, setCode] = useState('#include <stdio.h> int main() { printf("Hello World"); return 0; }');
    const [workspace, setWorkspace] = useState<String[]>(['23a629528f0e4437', '490e2609752840']);
    const [userId, setUserId] = useState<number>(1);
    const [workspaceList, setWorkspaceList] = useState< Workspace[]>([]);

    const currentCode = workspaceList[currentWorkspace]?.code || '';

    const deleteWorkspace = async (id: String) => {

        await axios.delete(`http://127.0.1.1:5000/delete_file/${id}`)
        await fetchData();
        if(workspaceList.length > 0){
            if(currentWorkspace)
                setCurrentWorkspace(currentWorkspace - 1);
        }
    }

    const createWorkspace = async () => {
        await axios.post('http://127.0.1.1:5000/create_file', {
            code: "#include <stdio.h> \n int main( { printf(\"Hello World33311\"); return 0; }"
        });
        await fetchData();
    }

    const executeCode = async () => {
        setText('');
        setIsLoading(true);
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
            axios.get(`http://127.0.1.1:5000/execute_file/${workspaceList[currentWorkspace].workspace_uid}`)
            .then((exec_response) => {
                const newText = exec_response.data.code_output;
                setText(newText);
                
            })
            .catch((exec_response) => {
                console.log(exec_response.response);
                const newText = exec_response.response.data.code_error;
                setText(newText);
            })
            .finally(()=>setIsLoading(false));
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
        setWorkspaceList(wl);
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
        setText('');
    }, [currentWorkspace, workspace.length])

    useEffect(()=> {
        const element = document.getElementById('loader');
        if(element) {
            if (isLoading) {
                element.style.visibility = 'visible';
            }else{
                element.style.visibility = 'hidden';
            }
        }
    }, [isLoading])


    return (
        <div className='container'>
            <div className='code-container'>
                <div className='control-panel'>
                    <button className='execute-button'
                    onClick={executeCode}
                    > Execute </button>
                </div>
                 <CodeMirror
                    id = 'code-area'
                    minWidth = '100%'
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
                <div className='output-container'>
                    <span className="loader" id='loader'></span>
                    <textarea 
                        value={text}
                    >
                    </textarea>
                </div>
            </div>
            <div className='work-spaces-container'>
                <div className='work-spaces'>
                    <div className='control-panel'>
                        <div className='control-panel'>
                            <button className='create-button'
                            onClick={createWorkspace}
                            > Create </button>
                        </div>
                    </div>
                    <div className="list-container">
                        <ul>
                            {workspaceList.map((workspace, index)=> {
                                return (
                                    <div className='list-item-container'>    
                                        <li 
                                            id={index.toString()} 
                                            onClick={()=>setCurrentWorkspace(index)
                                            }>{workspace.workspace_name}
                                        </li>
                                        <svg className="svg-cross" width="30" height="30" viewBox="0 0 24 24"
                                            onClick={()=>deleteWorkspace(workspace.workspace_uid)}
                                        >
                                            <line x1="2" y1="2" x2="22" y2="22" stroke="#000" stroke-width="2"/>
                                            <line x1="22" y1="2" x2="2" y2="22" stroke="#000" stroke-width="2"/>
                                        </svg>
                                    </div>
                                )
                            }) }
                        </ul>
                    </div>
                </div>
            </div>
        </div>
    )
}
