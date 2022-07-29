import { useEffect, useState } from "react";
import validators from "./validators";

// TODO: ENS
export function EthAddress(props: any) {
    const [value, setValue] = useState(props.defaultValue || "");
    const [valid, setValid] = useState(false);

    useEffect(() => {
        const valid_ = validators.isEthAddressValid(value);
        setValid(valid_);
        if(props.onValid) {
            props.onValid(valid_);
        }
    }, [value]);

    return <input value={value} onChange={(e: any) => { setValue(e.target.value); props.onChange(e); }} className={valid ? "" : "error"}/>
}