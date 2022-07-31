import React from "react";
import { useEffect, useState } from "react";
import validators from "./validators";

// TODO: ENS
export function EthAddress(props: {
    onValid?: (valid: boolean) => void,
    onChange?: (value: string) => void,
    value?: string,
    defaultValue?: string
}) {
    const [value, setValue] = useState(props.defaultValue || "");
    const [valid, setValid] = useState(false);

    useEffect(() => {
        const valid_ = validators.isEthAddressValid(value);
        setValid(valid_);
        if(props.onValid) {
            props.onValid(valid_);
        }
    }, [value]);

    useEffect(() => {
        if(props.value !== undefined) {
            setValue(props.value);
        }
    }, [props.value]);

    return <input type="text" defaultValue={props.defaultValue} className={valid ? "" : "error"}
        onChange={(e: any) => { setValue(e.target.value); if(props.onChange) { props.onChange(e); }}} />
}