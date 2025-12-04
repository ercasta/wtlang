# Keys and refs

Ref types allow creating references 

```
table Department {
    code: string key
    name: string
}


table Employee {
    global_id: string key
    name: string
    department: ref Department
    position: string
    salary: currency
}
```

we can get a table following the reference:

```
let employee = load_csv('Employees.csv', Employee)
let dep = employee.department
```

