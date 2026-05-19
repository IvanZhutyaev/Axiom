# AQL specification (summary)

AQL is a statically typed pipeline language. Stages are chained with `|>`.

## Example

```aql
source "sensor_data"
|> filter(temperature > 30.0)
|> window(tumbling, size=5s)
   aggregate(avg_temp = avg(temperature), count = count(*))
|> sink "alerts"
```

## Stage operators

`source`, `sink`, `filter`, `map`, `flatMap`, `keyBy`, `window`, `watermark`, `join`, `union`, `split`

## Types

Primitives: `int*`, `uint*`, `float*`, `bool`, `string`, `bytes`, `timestamp`  
Containers: `array<T>`, `map<K,V>`, `struct`, `T?`, `Stream<T>`, `Table<K,V>`

Full semantics: see project technical specification.
