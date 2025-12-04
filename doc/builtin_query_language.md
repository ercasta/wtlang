# Builtin query language

WTLang supports a simple embedded query language that also leverages common operators override

Examples:

```
let adults = users where age > 18
```

```
let male_adults = users where (age > 18 and sex = "male")
```

```
// minus operator
let other_users = users - male_adults
```

```
// union operator
let elders = users where age > 65
let children = users where age < 10

let special_tariff_users = elders + children
```

```
// intersect operator

let adults = users where age > 18
let male = users where sex = "male"

let male_adults = adults & male
```

```
// column subset selection
let users_main_data = users[name, surname]
```

```
// sort by. Asc and Desc are optional, default asc

let user_sorted_by_name_age = users sort by name asc, age desc
```


