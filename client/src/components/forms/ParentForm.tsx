import { ReactNode, useEffect, useState } from "react"
import './ParentForm.css'

type ModalProps = {
  children: ReactNode;
  isOpen: boolean; 
  setIsOpen: React.Dispatch<React.SetStateAction<boolean>>;
};

export const ParentForm: React.FC<ModalProps> = ({children, isOpen, setIsOpen}) => {
    
    useEffect(() => {
        const element = document.getElementById('form');
        if(element){
            if (isOpen) {
                element.style.visibility = 'visible';
            } else {
                element.style.visibility = 'hidden';
            }
        }
    }, [isOpen]);

    return (
        <div>
            <div id="form" className="background">
                <div className="content">
                    <svg className="svg-cross" width="30" height="30" viewBox="0 0 24 24"
                        onClick={() => setIsOpen(false)}
                    >
                        <line x1="2" y1="2" x2="22" y2="22" stroke="#000" stroke-width="2"/>
                        <line x1="22" y1="2" x2="2" y2="22" stroke="#000" stroke-width="2"/>
                    </svg>
                    {children}
                </div>
            </div>
        </div>
    )
}